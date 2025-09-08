// 增强版 database.rs - 集成了完整的错误处理机制
use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls};
use tracing::{info, warn, error};
use std::str::FromStr;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

/// 数据库管理器，基于Refinery实现，集成完整错误处理
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

    /// 执行安全迁移 - 增强版，集成完整错误处理
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
        .context("迁移任务执行失败");
        
        // 🆕 增强的错误处理 - 根据结果决定处理方式
        match migration_result {
            Ok(Ok(report)) => {
                // 处理成功结果
                info!("✅ Refinery数据库迁移完成");
                info!("已应用的迁移数量: {}", report.applied_migrations().len());
                
                for migration in report.applied_migrations() {
                    info!("  ✅ {}: {}", migration.version(), migration.name());
                }
                
                self.print_migration_status().await?;
                Ok(())
            }
            Ok(Err(migration_error)) => {
                // 🚨 迁移执行出错 - 触发完整的错误处理流程
                error!("❌ 迁移执行失败，启动错误处理流程...");
                error!("错误详情: {}", migration_error);
                
                // 执行错误处理流程
                if let Err(handle_err) = self.handle_migration_failure().await {
                    error!("⚠️  错误处理器本身发生错误: {}", handle_err);
                }
                
                Err(anyhow::anyhow!("迁移执行失败: {}", migration_error))
            }
            Err(task_error) => {
                // 🚨 异步任务执行出错
                error!("❌ 迁移任务执行失败，启动错误处理流程...");
                error!("任务错误详情: {}", task_error);
                
                // 执行错误处理流程
                if let Err(handle_err) = self.handle_migration_failure().await {
                    error!("⚠️  错误处理器本身发生错误: {}", handle_err);
                }
                
                Err(task_error)
            }
        }
    }
    
    /// 带重试机制的安全迁移
    pub async fn safe_migrate_with_retry(&mut self, database_url: &str, max_retries: u32) -> Result<()> {
        let mut attempts = 0;
        
        while attempts < max_retries {
            match self.safe_migrate(database_url).await {
                Ok(_) => {
                    if attempts > 0 {
                        info!("✅ 迁移在第 {} 次尝试后成功", attempts + 1);
                    }
                    return Ok(());
                }
                Err(e) => {
                    attempts += 1;
                    if attempts < max_retries {
                        let delay_seconds = 5 * attempts as u64;
                        warn!("⚠️  迁移失败，第 {} 次重试 (最大 {} 次)，{}秒后重试: {}", 
                              attempts, max_retries, delay_seconds, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds)).await;
                    } else {
                        error!("❌ 迁移在 {} 次尝试后仍然失败", max_retries);
                        return Err(e);
                    }
                }
            }
        }
        
        unreachable!()
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
    
    /// 🆕 处理迁移失败 - 现在被实际调用
    async fn handle_migration_failure(&self) -> Result<()> {
        error!("🚨 正在处理Refinery迁移失败...");
        
        // 生成详细的失败报告
        info!("📊 生成失败分析报告...");
        if let Err(e) = self.generate_failure_report().await {
            error!("生成失败报告时出错: {}", e);
        }
        
        // 检查数据完整性
        info!("🔍 检查数据完整性...");
        if let Err(e) = self.check_data_integrity().await {
            error!("数据完整性检查时出错: {}", e);
        }
        
        // 生成恢复建议
        self.generate_recovery_suggestions().await?;
        
        Ok(())
    }
    
    /// 🆕 生成失败报告 - 增强版
    async fn generate_failure_report(&self) -> Result<()> {
        let report_time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        
        error!("╔════════════════════════════════════════╗");
        error!("║        Refinery 迁移失败报告            ║");
        error!("╚════════════════════════════════════════╝");
        error!("📅 失败时间: {}", report_time);
        
        // 检查是否存在Refinery迁移历史表
        let history_exists = !self.client.query(
            "SELECT 1 FROM information_schema.tables WHERE table_name = 'refinery_schema_history'",
            &[]
        ).await?.is_empty();
        
        if history_exists {
            let migration_history = self.client.query(
                "SELECT version, name, applied_on, checksum 
                 FROM refinery_schema_history 
                 ORDER BY version DESC",
                &[]
            ).await?;
            
            error!("📊 迁移历史记录数量: {}", migration_history.len());
            error!("📋 最近的迁移记录:");
            
            for (i, row) in migration_history.iter().take(5).enumerate() {
                let version: i32 = row.get("version");
                let name: String = row.get("name");
                let applied_on: String = row.get("applied_on");
                let checksum: String = row.get("checksum");
                
                error!("  {}. V{:03}: {} ({})", 
                      i + 1, version, name, applied_on);
                error!("     校验和: {}", &checksum[..8]);
            }
        } else {
            error!("⚠️  Refinery迁移历史表不存在，这可能是首次迁移失败");
            error!("   或者数据库连接配置有问题");
        }
        
        // 检查文件系统中的迁移文件
        info!("📁 检查文件系统中的迁移文件...");
        // 注意：在实际环境中，这里需要实际读取文件系统
        error!("💾 文件系统状态将需要手动检查");
        
        error!("╚════════════════════════════════════════╝");
        
        Ok(())
    }
    
    /// 🆕 检查数据完整性 - 增强版
    async fn check_data_integrity(&self) -> Result<()> {
        info!("🔍 开始数据完整性检查...");
        
        let critical_tables = vec![
            ("users", "用户表"),
            ("products", "产品表"), 
            ("orders", "订单表"),
            ("refinery_schema_history", "迁移历史表")
        ];
        
        let mut integrity_issues = Vec::new();
        
        for (table, description) in critical_tables {
            info!("   检查 {} ({})...", table, description);
            
            // 检查表是否存在
            let exists_result = self.client.query(
                "SELECT EXISTS (
                    SELECT FROM information_schema.tables 
                    WHERE table_name = $1 AND table_schema = 'public'
                ) as exists",
                &[&table]
            ).await;
            
            match exists_result {
                Ok(rows) => {
                    let exists: bool = rows[0].get("exists");
                    
                    if exists {
                        // 检查表的数据量和基本结构
                        match self.check_table_details(table).await {
                            Ok(count) => {
                                info!("   ✅ {} 正常，包含 {} 行数据", description, count);
                            }
                            Err(e) => {
                                let issue = format!("表 {} 存在但查询失败: {}", table, e);
                                warn!("   ⚠️  {}", issue);
                                integrity_issues.push(issue);
                            }
                        }
                    } else {
                        let issue = format!("关键表 {} ({}) 不存在", table, description);
                        warn!("   ❌ {}", issue);
                        integrity_issues.push(issue);
                    }
                }
                Err(e) => {
                    let issue = format!("无法检查表 {} 的存在性: {}", table, e);
                    error!("   💥 {}", issue);
                    integrity_issues.push(issue);
                }
            }
        }
        
        // 汇总完整性检查结果
        if integrity_issues.is_empty() {
            info!("✅ 数据完整性检查通过，所有关键表都正常");
        } else {
            error!("❌ 发现 {} 个数据完整性问题:", integrity_issues.len());
            for (i, issue) in integrity_issues.iter().enumerate() {
                error!("   {}. {}", i + 1, issue);
            }
        }
        
        Ok(())
    }
    
    /// 检查单个表的详细信息
    async fn check_table_details(&self, table_name: &str) -> Result<i64> {
        let rows = self.client.query(
            &format!("SELECT COUNT(*) as count FROM {}", table_name),
            &[]
        ).await?;
        
        let count: i64 = rows[0].get("count");
        Ok(count)
    }
    
    /// 🆕 生成恢复建议
    async fn generate_recovery_suggestions(&self) -> Result<()> {
        error!("╔════════════════════════════════════════╗");
        error!("║           恢复建议和后续步骤            ║");
        error!("╚════════════════════════════════════════╝");
        error!("");
        error!("🔧 建议的恢复步骤:");
        error!("   1. 检查错误日志以确定具体的失败原因");
        error!("   2. 验证数据库连接和权限设置");
        error!("   3. 检查迁移文件的SQL语法");
        error!("   4. 考虑从最近的备份恢复（如果有）");
        error!("   5. 手动修复数据库状态后重新运行迁移");
        error!("");
        error!("🚨 立即行动项:");
        error!("   • 不要删除任何数据");
        error!("   • 创建当前数据库状态的备份");
        error!("   • 联系数据库管理员（如果适用）");
        error!("");
        error!("📞 如需帮助，请查阅 Refinery 使用指南或联系技术支持");
        error!("╚════════════════════════════════════════╝");
        
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

/// 🆕 增强的连接函数，支持重试
pub async fn connect_with_retry(max_retries: u32) -> Result<DatabaseManager> {
    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://sxt:default@localhost:5432/postgres".to_string());
    
    let mut attempts = 0;
    
    while attempts < max_retries {
        match DatabaseManager::new_with_config(&database_url).await {
            Ok(manager) => {
                if attempts > 0 {
                    info!("✅ 数据库连接在第 {} 次尝试后成功", attempts + 1);
                }
                return Ok(manager);
            }
            Err(e) => {
                attempts += 1;
                if attempts < max_retries {
                    let delay_seconds = 3 * attempts as u64;
                    warn!("⚠️  数据库连接失败，第 {} 次重试 (最大 {} 次)，{}秒后重试: {}", 
                          attempts, max_retries, delay_seconds, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds)).await;
                } else {
                    error!("❌ 数据库连接在 {} 次尝试后仍然失败: {}", max_retries, e);
                    return Err(e);
                }
            }
        }
    }
    
    unreachable!()
}
