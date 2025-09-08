// 双连接架构示例代码
// 展示同步连接（迁移）和异步连接（业务逻辑）的实际使用

use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls};
use tracing::{info, error};
use std::str::FromStr;

// 嵌入迁移文件
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub struct DatabaseManager {
    // 这是异步连接，用于业务逻辑
    pub async_client: Client,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        info!("🔌 创建异步数据库连接（用于业务逻辑）");
        
        // 创建异步连接 - 使用 tokio-postgres
        let (async_client, connection) = tokio_postgres::connect(database_url, NoTls)
            .await
            .context("创建异步连接失败")?;
        
        // 在后台维护连接
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("异步连接错误: {}", e);
            }
        });
        
        Ok(Self { async_client })
    }

    /// 执行数据库迁移（使用同步连接）
    pub async fn migrate(&self, database_url: &str) -> Result<()> {
        info!("🔄 开始数据库迁移（将创建临时同步连接）");
        
        let database_url = database_url.to_owned();
        
        // 在专用的阻塞线程中执行迁移
        let report = tokio::task::spawn_blocking(move || -> Result<refinery::Report> {
            info!("🔌 创建同步数据库连接（仅用于迁移）");
            
            // 创建同步连接 - 使用 postgres（注意不是tokio-postgres）
            let config = postgres::Config::from_str(&database_url)
                .context("解析数据库URL失败")?;
            
            let mut sync_client = config.connect(postgres::NoTls)
                .context("创建同步连接失败")?;
            
            info!("📋 执行迁移...");
            
            // Refinery 只能使用同步连接
            let report = embedded::migrations::runner().run(&mut sync_client)
                .map_err(|e| anyhow::anyhow!("迁移执行失败: {}", e))?;
            
            info!("✅ 迁移完成，同步连接即将关闭");
            // sync_client 在这里自动释放
            
            Ok(report)
        }).await
        .context("迁移任务执行失败")?
        .context("迁移失败")?;
        
        let applied = report.applied_migrations();
        if !applied.is_empty() {
            info!("📊 应用了 {} 个迁移:", applied.len());
            for migration in applied {
                info!("  ✅ V{}: {}", migration.version(), migration.name());
            }
        } else {
            info!("📊 数据库已是最新版本，无需迁移");
        }
        
        Ok(())
    }

    /// 业务逻辑：获取用户列表（使用异步连接）
    pub async fn get_users(&self) -> Result<Vec<User>> {
        info!("🔍 查询用户列表（使用异步连接）");
        
        // 使用异步连接进行查询
        let rows = self.async_client.query(
            "SELECT id, username, email, created_at FROM users ORDER BY id",
            &[]
        ).await
        .context("查询用户失败")?;
        
        let users: Vec<User> = rows.iter().map(|row| User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            created_at: row.get("created_at"),
        }).collect();
        
        info!("📋 查询到 {} 个用户", users.len());
        Ok(users)
    }

    /// 业务逻辑：创建用户（使用异步连接）
    pub async fn create_user(&self, username: &str, email: &str) -> Result<i32> {
        info!("➕ 创建用户: {} (使用异步连接)", username);
        
        // 使用异步连接进行插入
        let row = self.async_client.query_one(
            "INSERT INTO users (username, email, password_hash, full_name) 
             VALUES ($1, $2, 'temp_hash', $1) 
             RETURNING id",
            &[&username, &email]
        ).await
        .context("创建用户失败")?;
        
        let user_id: i32 = row.get("id");
        info!("✅ 用户创建成功，ID: {}", user_id);
        Ok(user_id)
    }
}

#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 启动应用程序 - 双连接架构演示");
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://sxt:default@localhost:5432/postgres".to_string());
    
    // ==================== 阶段 1: 创建数据库管理器 ====================
    info!("📝 阶段 1: 创建数据库管理器（异步连接）");
    let db_manager = DatabaseManager::new(&database_url).await
        .context("创建数据库管理器失败")?;
    
    // ==================== 阶段 2: 执行数据库迁移 ====================
    info!("📝 阶段 2: 执行数据库迁移（临时同步连接）");
    db_manager.migrate(&database_url).await
        .context("数据库迁移失败")?;
    
    // ==================== 阶段 3: 运行业务逻辑 ====================
    info!("📝 阶段 3: 运行业务逻辑（使用异步连接）");
    
    // 创建一个测试用户
    let user_id = db_manager.create_user("demo_user", "demo@example.com").await?;
    info!("创建的用户ID: {}", user_id);
    
    // 查询所有用户
    let users = db_manager.get_users().await?;
    for user in users {
        info!("用户: {} <{}> (ID: {})", user.username, user.email, user.id);
    }
    
    // ==================== 总结 ====================
    info!("📊 双连接架构总结:");
    info!("  🔧 迁移阶段: 使用临时同步连接 (postgres::Client)");
    info!("     - 在 spawn_blocking 中创建");
    info!("     - 执行完迁移后自动释放");
    info!("     - 简单可靠，适合一次性操作");
    info!("  🚀 业务阶段: 使用长期异步连接 (tokio_postgres::Client)");
    info!("     - 在应用启动时创建");
    info!("     - 整个应用生命周期中保持连接");
    info!("     - 高性能，支持并发查询");
    
    info!("🎉 双连接架构演示完成！");
    Ok(())
}

/* 
Cargo.toml 依赖配置:

[dependencies]
# 核心异步运行时
tokio = { version = "1.0", features = ["full"] }

# 同步连接 - 仅用于迁移
postgres = { version = "0.19", features = ["with-chrono-0_4"] }

# 异步连接 - 用于业务逻辑  
tokio-postgres = { version = "0.7", features = ["with-chrono-0_4"] }

# 迁移框架
refinery = { version = "0.8", features = ["postgres"] }

# 辅助库
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
chrono = { version = "0.4", features = ["serde"] }
dotenv = "0.15"

运行输出示例:
🚀 启动应用程序 - 双连接架构演示
📝 阶段 1: 创建数据库管理器（异步连接）
🔌 创建异步数据库连接（用于业务逻辑）
📝 阶段 2: 执行数据库迁移（临时同步连接）
🔄 开始数据库迁移（将创建临时同步连接）
🔌 创建同步数据库连接（仅用于迁移）
📋 执行迁移...
✅ 迁移完成，同步连接即将关闭
📊 数据库已是最新版本，无需迁移
📝 阶段 3: 运行业务逻辑（使用异步连接）
➕ 创建用户: demo_user (使用异步连接)
✅ 用户创建成功，ID: 1
🔍 查询用户列表（使用异步连接）
📋 查询到 1 个用户
用户: demo_user <demo@example.com> (ID: 1)
🎉 双连接架构演示完成！
*/
