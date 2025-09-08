use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls};
use tracing::{info, warn, error};
use std::str::FromStr;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

/// 数据库管理器，基于Refinery实现
pub struct DatabaseManager {
    pub client: Client,
}

impl DatabaseManager {
    /// 创建具有自定义配置的数据库连接
    pub async fn new_with_config(database_url: &str) -> Result<Self> {
        info!("正在使用Refinery连接数据库...");
        
        // 解析数据库URL并建立连接
        let (client, connection) = tokio_postgres::connect(database_url, NoTls)
            .await
            .context("无法连接到数据库")?;
        
        // 在后台运行连接
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("数据库连接错误: {}", e);
            }
        });
        
        // 测试连接
        let _row = client.query_one("SELECT 1", &[])
            .await
            .context("数据库连接测试失败")?;
        
        info!("数据库连接验证成功");
        Ok(Self { client })
    }

    /// 执行安全迁移
    pub async fn safe_migrate(&mut self, database_url: &str) -> Result<()> {
        info!("开始执行Refinery数据库迁移...");
        
        // 检查是否为现有数据库
        if self.is_existing_database().await? {
            info!("检测到现有数据库，建立迁移基线...");
            self.setup_baseline_for_existing_db().await?;
        }
        
        // 在单独的线程中执行同步的迁移操作，避免运行时冲突
        let database_url_owned = database_url.to_owned();
        let migration_result = tokio::task::spawn_blocking(move || -> Result<refinery::Report, anyhow::Error> {
            // 使用同步的 postgres 客户端进行迁移
            let db_config = postgres::Config::from_str(&database_url_owned)
                .context("解析数据库URL失败")?;
            
            let mut postgres_client = db_config.connect(postgres::NoTls)
                .context("创建迁移专用连接失败")?;
            
            // 执行迁移
            let report = embedded::migrations::runner().run(&mut postgres_client)
                .map_err(|e| anyhow::anyhow!("Refinery迁移执行失败: {}", e))?;
            
            Ok(report)
        }).await
        .context("迁移任务执行失败")?
        .context("迁移操作失败")?;
        
        // 处理迁移结果
        info!("✅ Refinery数据库迁移完成");
        info!("已应用的迁移数量: {}", migration_result.applied_migrations().len());
        
        for migration in migration_result.applied_migrations() {
            info!("  ✅ {}: {}", migration.version(), migration.name());
        }
        
        self.print_migration_status().await?;
        Ok(())
    }
    
    /// 检查是否为现有数据库
    async fn is_existing_database(&self) -> Result<bool> {
        let rows = self.client.query(
            "SELECT COUNT(*) as count FROM information_schema.tables 
             WHERE table_schema = 'public' AND table_type = 'BASE TABLE'",
            &[]
        ).await?;
        
        let table_count: i64 = rows[0].get("count");
        Ok(table_count > 0)
    }
    
    /// 为现有数据库设置迁移基线
    async fn setup_baseline_for_existing_db(&self) -> Result<()> {
        // 检查是否已经有Refinery迁移记录表
        let rows = self.client.query(
            "SELECT 1 FROM information_schema.tables WHERE table_name = 'refinery_schema_history'",
            &[]
        ).await?;
        
        if rows.is_empty() {
            info!("首次为现有数据库设置Refinery迁移基线...");
            info!("Refinery将自动创建schema_history表并记录迁移状态");
        } else {
            info!("Refinery迁移基线已存在");
        }
        
        Ok(())
    }
    
    /// 处理迁移失败
    async fn handle_migration_failure(&self) -> Result<()> {
        error!("正在处理Refinery迁移失败...");
        
        // 生成失败报告
        self.generate_failure_report().await?;
        
        // 检查数据完整性
        self.check_data_integrity().await?;
        
        Ok(())
    }
    
    /// 生成失败报告
    async fn generate_failure_report(&self) -> Result<()> {
        let report_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        
        // 检查是否存在Refinery迁移历史表
        let rows = self.client.query(
            "SELECT 1 FROM information_schema.tables WHERE table_name = 'refinery_schema_history'",
            &[]
        ).await?;
        
        if !rows.is_empty() {
            let migration_history = self.client.query(
                "SELECT version, name, applied_on, checksum 
                 FROM refinery_schema_history 
                 ORDER BY version DESC",
                &[]
            ).await?;
            
            error!("=== Refinery迁移失败报告 ===");
            error!("时间: {}", report_time);
            error!("迁移历史记录数量: {}", migration_history.len());
            
            for row in migration_history.iter().take(5) { // 只显示最近5个
                let version: i32 = row.get("version");
                let name: String = row.get("name");
                let applied_on: chrono::DateTime<chrono::Utc> = row.get("applied_on");
                error!("  - v{}: {} ({})", version, name, applied_on.format("%Y-%m-%d %H:%M:%S"));
            }
        } else {
            error!("Refinery迁移历史表不存在，可能是首次迁移失败");
        }
        
        Ok(())
    }
    
    /// 检查数据完整性
    async fn check_data_integrity(&self) -> Result<()> {
        info!("检查数据完整性...");
        
        let critical_tables = vec!["users", "products", "orders"];
        
        for table in critical_tables {
            let rows = self.client.query(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_name = $1 AND table_schema = 'public'
                ) as exists",
                &[&table]
            ).await?;
            
            let exists: bool = rows[0].get("exists");
            
            if exists {
                let count_rows = self.client.query(
                    &format!("SELECT COUNT(*) as count FROM {}", table),
                    &[]
                ).await;
                
                match count_rows {
                    Ok(rows) => {
                        let count: i64 = rows[0].get("count");
                        info!("✅ 表 {} 存在，包含 {} 行数据", table, count);
                    }
                    Err(e) => {
                        warn!("⚠️  无法查询表 {} 的数据量: {}", table, e);
                    }
                }
            } else {
                warn!("⚠️  关键表 {} 不存在", table);
            }
        }
        
        Ok(())
    }
    
    /// 打印迁移状态
    async fn print_migration_status(&self) -> Result<()> {
        let rows = self.client.query(
            "SELECT version, name, applied_on, checksum 
             FROM refinery_schema_history 
             ORDER BY version",
            &[]
        ).await;
        
        match rows {
            Ok(migrations) => {
                info!("\n=== 当前Refinery迁移状态 ===");
                for row in migrations {
                    let version: i32 = row.get("version");
                    let name: String = row.get("name");
                    let applied_on: String = row.get("applied_on");
                    
                    info!("  ✅ v{:03}: {} ({})", 
                          version, 
                          name,
                          applied_on);
                }
                info!("");
            }
            Err(e) => {
                warn!("无法获取迁移状态: {}", e);
            }
        }
        
        Ok(())
    }
    
    /// 获取客户端引用
    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

/// 便捷函数：使用默认DATABASE_URL环境变量创建数据库管理器
pub async fn connect() -> Result<DatabaseManager> {
    dotenv::dotenv().ok(); // 加载环境变量，如果文件存在的话
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://sxt:default@localhost:5432/postgres".to_string());
    
    DatabaseManager::new_with_config(&database_url).await
}
