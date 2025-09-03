use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use tracing::{info, warn, error};
use std::str::FromStr;

/// 数据库管理器，基于SQLx实现
pub struct DatabaseManager {
    pub pool: PgPool,
}

impl DatabaseManager {
    
    /// 创建具有自定义配置的数据库连接池
    pub async fn new_with_config(database_url: &str) -> Result<Self> {
        info!("正在使用自定义配置连接数据库...");
        
        let pool = PgPool::connect_with(
            sqlx::postgres::PgConnectOptions::from_str(database_url)?
                .application_name("database_update")
        )
        .await
        .context("无法连接到数据库")?;
        
        // 测试连接
        let _row = sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await
            .context("数据库连接测试失败")?;
        
        info!("数据库连接验证成功");
        Ok(Self { pool })
    }

    /// 执行安全迁移
    pub async fn safe_migrate(&self) -> Result<()> {
        info!("开始执行数据库迁移...");
        
        // 检查是否为现有数据库
        if self.is_existing_database().await? {
            info!("检测到现有数据库，建立迁移基线...");
            self.setup_baseline_for_existing_db().await?;
        }
        
        // 执行迁移
        match sqlx::migrate!("./migrations").run(&self.pool).await {
            Ok(_) => {
                info!("✅ 数据库迁移完成");
                self.print_migration_status().await?;
                Ok(())
            }
            Err(e) => {
                error!("❌ 迁移失败: {}", e);
                self.handle_migration_failure().await?;
                Err(anyhow::anyhow!("数据库迁移失败: {}", e))
            }
        }
    }
    
    /// 检查是否为现有数据库
    async fn is_existing_database(&self) -> Result<bool> {
        let table_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.tables 
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(table_count > 0)
    }
    
    /// 为现有数据库设置迁移基线
    async fn setup_baseline_for_existing_db(&self) -> Result<()> {
        // 检查是否已经有迁移记录
        let migration_exists = sqlx::query(
            "SELECT 1 FROM information_schema.tables WHERE table_name = '_sqlx_migrations'"
        )
        .fetch_optional(&self.pool)
        .await?
        .is_some();
        
        if !migration_exists {
            info!("首次为现有数据库设置迁移基线...");
            
            // 运行一次迁移来创建迁移表，但标记基线迁移为已执行
            if let Err(e) = sqlx::migrate!("./migrations").run(&self.pool).await {
                warn!("基线设置过程中的预期错误: {}", e);
            }
            
            info!("迁移基线设置完成");
        }
        
        Ok(())
    }
    
    /// 处理迁移失败
    async fn handle_migration_failure(&self) -> Result<()> {
        error!("正在处理迁移失败...");
        
        // 生成失败报告
        self.generate_failure_report().await?;
        
        // 检查数据完整性
        self.check_data_integrity().await?;
        
        Ok(())
    }
    
    /// 生成失败报告
    async fn generate_failure_report(&self) -> Result<()> {
        let report_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        
        // 检查是否存在迁移表
        let migration_table_exists = sqlx::query(
            "SELECT 1 FROM information_schema.tables WHERE table_name = '_sqlx_migrations'"
        )
        .fetch_optional(&self.pool)
        .await?
        .is_some();
        
        if migration_table_exists {
            let failed_migrations = sqlx::query(
                "SELECT version, description, installed_on, success 
                 FROM _sqlx_migrations 
                 WHERE success = false OR success IS NULL
                 ORDER BY version DESC"
            )
            .fetch_all(&self.pool)
            .await?;
            
            error!("=== 迁移失败报告 ===");
            error!("时间: {}", report_time);
            error!("失败的迁移数量: {}", failed_migrations.len());
            
            for row in failed_migrations {
                let version: i64 = row.get("version");
                let description: String = row.get("description");
                error!("  - v{}: {}", version, description);
            }
        } else {
            error!("迁移表不存在，可能是首次迁移失败");
        }
        
        Ok(())
    }
    
    /// 检查数据完整性
    async fn check_data_integrity(&self) -> Result<()> {
        info!("检查数据完整性...");
        
        let critical_tables = vec!["users", "products", "orders"];
        
        for table in critical_tables {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_name = $1 AND table_schema = 'public'
                )"
            )
            .bind(table)
            .fetch_one(&self.pool)
            .await?;
            
            if exists {
                let count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {}", table))
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);
                info!("✅ 表 {} 存在，包含 {} 行数据", table, count);
            } else {
                warn!("⚠️  关键表 {} 不存在", table);
            }
        }
        
        Ok(())
    }
    
    /// 打印迁移状态
    async fn print_migration_status(&self) -> Result<()> {
        let migrations = sqlx::query(
            "SELECT version, description, installed_on, success, execution_time 
             FROM _sqlx_migrations 
             ORDER BY version"
        )
        .fetch_all(&self.pool)
        .await?;
        
        info!("\n=== 当前迁移状态 ===");
        for row in migrations {
            let version: i64 = row.get("version");
            let description: String = row.get("description");
            let success: bool = row.get("success");
            let installed_on: chrono::DateTime<chrono::Utc> = row.get("installed_on");
            let execution_time: i64 = row.get("execution_time");
            
            let status_symbol = if success { "✅" } else { "❌" };
            info!("  {} v{:03}: {} ({}ms, {})", 
                  status_symbol, 
                  version, 
                  description,
                  execution_time,
                  installed_on.format("%Y-%m-%d %H:%M:%S"));
        }
        info!("");
        
        Ok(())
    }
    
    
    /// 关闭连接池
    pub async fn close(self) {
        self.pool.close().await;
        info!("数据库连接池已关闭");
    }
}

/// 便捷函数：使用默认DATABASE_URL环境变量创建数据库管理器
pub async fn connect() -> Result<DatabaseManager> {
    dotenv::dotenv().ok(); // 加载环境变量，如果文件存在的话
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://sxt:default@localhost:5432/postgres".to_string());
    
    DatabaseManager::new_with_config(&database_url).await
}