# SQLx 项目集成完整指南

## 🚀 概述

SQLx 是一个 Rust 异步 SQL 工具包，支持多种数据库（PostgreSQL、MySQL、SQLite、SQL Server），提供编译时 SQL 查询检查、类型安全和零成本抽象。

## 📋 在新项目中引入 SQLx 的完整步骤

### 1. 项目初始化

#### 1.1 创建新项目
```bash
cargo new my_sqlx_project
cd my_sqlx_project
```

#### 1.2 配置 Cargo.toml
```toml
[package]
name = "my_sqlx_project"
version = "0.1.0"
edition = "2021"

[dependencies]
# 异步运行时 - SQLx 必需
tokio = { version = "1.0", features = ["full"] }
# SQLx 核心 - 数据库连接和迁移必需
sqlx = { version = "0.8", features = [
    "runtime-tokio-rustls",  # 异步运行时
    "postgres",              # PostgreSQL 支持
    "chrono",                # 时间类型支持
    "uuid",                  # UUID 类型支持
    "migrate"                # 数据库迁移支持
]}
# 环境变量 - 用于配置数据库连接
dotenv = "0.15"
```
#### 1.3 安装 sqlx-cli

cargo install sqlx-cli --features postgres

### 2. 环境配置

#### 2.1 创建 .env 文件
```bash
# 数据库连接配置
DATABASE_URL=postgresql://用户名:密码@主机:端口/数据库名

# 示例：
DATABASE_URL=postgresql://sxt:default@localhost:5432/postgres

# 日志级别
RUST_LOG=info

# 迁移失败处理策略
MIGRATION_FAILURE_STRATEGY=manual
```

#### 2.2 数据库连接配置
```rust
// src/database.rs
use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use tracing::info;

pub struct DatabaseManager {
    pub pool: PgPool,
}

impl DatabaseManager {
    /// 创建数据库连接池
    pub async fn new_with_config(database_url: &str) -> Result<Self> {
        info!("正在连接数据库...");
        
        let pool = PgPool::connect_with(
            sqlx::postgres::PgConnectOptions::from_str(database_url)?
                .application_name("my_sqlx_project")
        )
        .await
        .context("无法连接到数据库")?;
        
        // 测试连接
        let _row = sqlx::query("SELECT 1")
            .fetch_one(&pool)
            .await
            .context("数据库连接测试失败")?;
        
        info!("数据库连接成功");
        Ok(Self { pool })
    }
}

/// 便捷连接函数
pub async fn connect() -> Result<DatabaseManager> {
    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://sxt:default@localhost:5432/postgres".to_string());
    
    DatabaseManager::new_with_config(&database_url).await
}
```

### 3. 数据库迁移系统

#### 3.1 使用 sqlx-cli 管理迁移
```bash
# 创建新的迁移文件
sqlx migrate add create_users_table
# 若已有迁移文件 001_initial_schema.sql，新创建的文件名格式会跟随已有文件 002_create_users_table.sql
# 若没有，会采用默认命名方式 20250903090034_create_users_table.sql

# 运行所有待处理的迁移
sqlx migrate run

# 回滚最后一个迁移（一般不用，执行新的迁移覆盖之前的迁移）
sqlx migrate revert

# 查看迁移状态
sqlx migrate info

# 创建迁移目录（如果不存在）
mkdir migrations
```
sqlx migrate run 是一个独立的命令行工具操作，它：
直接读取迁移文件 - 从 migrations/ 目录读取 .sql 文件
直接连接数据库 - 使用 DATABASE_URL 环境变量连接数据库
执行SQL语句 - 直接在数据库中执行迁移SQL
更新迁移表 - 在 _sqlx_migrations 表中记录执行状态
这个过程完全独立于Rust代码编译。

运行cargo run也会执行迁移（在main.rs中使用了sqlx::migrate!()宏），但是需要重新编译，迁移才会成功。因为：
宏在编译时读取迁移文件
迁移文件的内容会被编译到二进制文件中
新的迁移文件需要重新编译才能被包含

#### 3.2 迁移文件命名规则
```
<版本号>_<描述>.sql

示例：
migrations/
├── 001_initial_schema.sql           # version=1, description="initial schema"
├── 002_add_user_preferences.sql     # version=2, description="add user preferences"
├── 003_create_products_table.sql    # version=3, description="create products table"
└── 20231201_add_orders_table.sql    # version=20231201, description="add orders table"
```

#### 3.3 迁移文件示例
```sql
-- 001_initial_schema.sql
-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN DEFAULT TRUE
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
```

#### 3.4 迁移执行代码
```rust
// src/database.rs
impl DatabaseManager {
    /// 执行数据库迁移
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
}
```

#### 迁移表 _sqlx_migrations
```
CREATE TABLE _sqlx_migrations ( 
    version BIGINT PRIMARY KEY,        -- 从文件名解析的版本号
    description TEXT NOT NULL,         -- 从文件名解析的描述
    installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- 安装时间
    success BOOLEAN NOT NULL,          -- 执行是否成功
    checksum BYTEA NOT NULL,          -- 文件内容校验和
    execution_time BIGINT NOT NULL    -- 执行耗时（纳秒）
);
```
### 4. 数据模型和查询

#### 4.1 定义数据模型
```rust
// src/models.rs
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub price: Decimal,
    pub category_id: Option<i32>,
    pub stock_quantity: i32,
    pub sku: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_active: bool,
}
```

#### 4.2 基础查询操作
```rust
// src/data.rs
use anyhow::{Context, Result};
use sqlx::{PgPool, Row};
use tracing::info;

/// 查询所有用户
pub async fn get_all_users(pool: &PgPool) -> Result<Vec<User>> {
    let rows = sqlx::query(
        "SELECT id, username, email, password_hash, full_name, 
                created_at, updated_at, is_active 
         FROM users 
         WHERE is_active = true 
         ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
    .context("查询用户失败")?;
    
    let users = rows.into_iter().map(|row| User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        full_name: row.get("full_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        is_active: row.get("is_active"),
    }).collect();
    
    Ok(users)
}

/// 根据ID查询用户
pub async fn get_user_by_id(pool: &PgPool, user_id: i32) -> Result<Option<User>> {
    let row = sqlx::query(
        "SELECT id, username, email, password_hash, full_name, 
                created_at, updated_at, is_active 
         FROM users 
         WHERE id = $1 AND is_active = true"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .context("查询用户失败")?;
    
    match row {
        Some(row) => Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            full_name: row.get("full_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        })),
        None => Ok(None),
    }
}

/// 创建新用户
pub async fn create_user(
    pool: &PgPool, 
    username: &str, 
    email: &str, 
    password_hash: &str, 
    full_name: Option<&str>
) -> Result<User> {
    let row = sqlx::query(
        "INSERT INTO users (username, email, password_hash, full_name) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, username, email, password_hash, full_name, 
                   created_at, updated_at, is_active"
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(full_name)
    .fetch_one(pool)
    .await
    .context("创建用户失败")?;
    
    Ok(User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        full_name: row.get("full_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        is_active: row.get("is_active"),
    })
}

/// 更新用户信息
pub async fn update_user(
    pool: &PgPool, 
    user_id: i32, 
    updates: &UserUpdate
) -> Result<Option<User>> {
    let mut query_parts = Vec::new();
    let mut bind_values = Vec::new();
    let mut param_count = 1;
    
    if let Some(username) = &updates.username {
        query_parts.push(format!("username = ${}", param_count));
        bind_values.push(username);
        param_count += 1;
    }
    
    if let Some(email) = &updates.email {
        query_parts.push(format!("email = ${}", param_count));
        bind_values.push(email);
        param_count += 1;
    }
    
    if let Some(full_name) = &updates.full_name {
        query_parts.push(format!("full_name = ${}", param_count));
        bind_values.push(full_name);
        param_count += 1;
    }
    
    if query_parts.is_empty() {
        return Ok(None);
    }
    
    query_parts.push("updated_at = CURRENT_TIMESTAMP".to_string());
    
    let query = format!(
        "UPDATE users SET {} WHERE id = ${} AND is_active = true 
         RETURNING id, username, email, password_hash, full_name, 
                   created_at, updated_at, is_active",
        query_parts.join(", "), param_count
    );
    
    let mut query_builder = sqlx::query(&query);
    for value in bind_values {
        query_builder = query_builder.bind(value);
    }
    query_builder = query_builder.bind(user_id);
    
    let row = query_builder
        .fetch_optional(pool)
        .await
        .context("更新用户失败")?;
    
    match row {
        Some(row) => Ok(Some(User {
            id: row.get("id"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            full_name: row.get("full_name"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            is_active: row.get("is_active"),
        })),
        None => Ok(None),
    }
}

/// 删除用户（软删除）
pub async fn delete_user(pool: &PgPool, user_id: i32) -> Result<bool> {
    let result = sqlx::query(
        "UPDATE users SET is_active = false, updated_at = CURRENT_TIMESTAMP 
         WHERE id = $1 AND is_active = true"
    )
    .bind(user_id)
    .execute(pool)
    .await
    .context("删除用户失败")?;
    
    Ok(result.rows_affected() > 0)
}
```

#### 4.3 复杂查询示例
```rust
/// 分页查询用户
pub async fn get_users_paginated(
    pool: &PgPool, 
    page: i64, 
    page_size: i64,
    search: Option<&str>
) -> Result<(Vec<User>, i64)> {
    let offset = (page - 1) * page_size;
    
    // 构建搜索条件
    let search_condition = if let Some(search_term) = search {
        "WHERE (username ILIKE $1 OR email ILIKE $1 OR full_name ILIKE $1) AND is_active = true"
    } else {
        "WHERE is_active = true"
    };
    
    // 查询总数
    let count_query = if search.is_some() {
        "SELECT COUNT(*) FROM users WHERE (username ILIKE $1 OR email ILIKE $1 OR full_name ILIKE $1) AND is_active = true"
    } else {
        "SELECT COUNT(*) FROM users WHERE is_active = true"
    };
    
    let total_count: i64 = if let Some(search_term) = search {
        sqlx::query_scalar(count_query)
            .bind(format!("%{}%", search_term))
            .fetch_one(pool)
            .await?
    } else {
        sqlx::query_scalar(count_query)
            .fetch_one(pool)
            .await?
    };
    
    // 查询数据
    let data_query = format!(
        "SELECT id, username, email, password_hash, full_name, 
                created_at, updated_at, is_active 
         FROM users 
         {} 
         ORDER BY created_at DESC 
         LIMIT $1 OFFSET $2",
        search_condition
    );
    
    let rows = if let Some(search_term) = search {
        sqlx::query(&data_query)
            .bind(format!("%{}%", search_term))
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query(&data_query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await?
    };
    
    let users = rows.into_iter().map(|row| User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        full_name: row.get("full_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        is_active: row.get("is_active"),
    }).collect();
    
    Ok((users, total_count))
}

/// 统计查询
pub async fn get_user_statistics(pool: &PgPool) -> Result<UserStatistics> {
    let stats = sqlx::query(
        "SELECT 
            COUNT(*) as total_users,
            COUNT(CASE WHEN created_at >= CURRENT_DATE - INTERVAL '30 days' THEN 1 END) as new_users_30d,
            COUNT(CASE WHEN created_at >= CURRENT_DATE - INTERVAL '7 days' THEN 1 END) as new_users_7d,
            COUNT(CASE WHEN created_at >= CURRENT_DATE THEN 1 END) as new_users_today
         FROM users 
         WHERE is_active = true"
    )
    .fetch_one(pool)
    .await
    .context("获取用户统计失败")?;
    
    Ok(UserStatistics {
        total_users: stats.get("total_users"),
        new_users_30d: stats.get("new_users_30d"),
        new_users_7d: stats.get("new_users_7d"),
        new_users_today: stats.get("new_users_today"),
    })
}
```

### 5. 主程序集成

#### 5.1 主程序示例
```rust
// src/main.rs
mod database;
mod models;
mod data;

use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 启动 SQLx 项目");
    
    // 加载环境变量
    dotenv::dotenv().ok();
    
    // 连接数据库
    let db_manager = match database::connect().await {
        Ok(manager) => {
            info!("✅ 数据库连接成功");
            manager
        }
        Err(e) => {
            error!("❌ 数据库连接失败: {}", e);
            return Err(e);
        }
    };
    
    // 执行数据库迁移
    info!("📋 开始执行数据库迁移...");
    match db_manager.safe_migrate().await {
        Ok(_) => {
            info!("✅ 数据库迁移完成");
        }
        Err(e) => {
            error!("❌ 数据库迁移失败: {}", e);
            return Err(e);
        }
    }
    
    // 示例：创建用户
    info!("👤 创建示例用户...");
    match data::create_user(
        &db_manager.pool,
        "testuser",
        "test@example.com",
        "hashed_password",
        Some("测试用户")
    ).await {
        Ok(user) => {
            info!("✅ 用户创建成功: {}", user.username);
        }
        Err(e) => {
            error!("❌ 用户创建失败: {}", e);
        }
    }
    
    // 示例：查询用户
    info!("🔍 查询用户列表...");
    match data::get_all_users(&db_manager.pool).await {
        Ok(users) => {
            info!("✅ 查询到 {} 个用户", users.len());
            for user in users {
                info!("  - {} ({})", user.username, user.email);
            }
        }
        Err(e) => {
            error!("❌ 查询用户失败: {}", e);
        }
    }
    
    // 示例：分页查询
    info!("📄 分页查询用户...");
    match data::get_users_paginated(&db_manager.pool, 1, 10, None).await {
        Ok((users, total)) => {
            info!("✅ 分页查询成功: 第1页，共{}个用户，总数{}", users.len(), total);
        }
        Err(e) => {
            error!("❌ 分页查询失败: {}", e);
        }
    }
    
    info!("🎉 程序执行完成");
    
    // 关闭数据库连接
    db_manager.close().await;
    
    Ok(())
}
```

### 6. 高级特性

#### 6.1 事务处理
```rust
/// 使用事务创建用户和订单
pub async fn create_user_with_order(
    pool: &PgPool,
    user_data: &UserCreate,
    order_data: &OrderCreate
) -> Result<(User, Order)> {
    let mut tx = pool.begin().await?;
    
    // 在事务中创建用户
    let user = create_user_in_transaction(&mut tx, user_data).await?;
    
    // 在事务中创建订单
    let order = create_order_in_transaction(&mut tx, &user.id, order_data).await?;
    
    // 提交事务
    tx.commit().await?;
    
    Ok((user, order))
}

async fn create_user_in_transaction(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    user_data: &UserCreate
) -> Result<User> {
    let row = sqlx::query(
        "INSERT INTO users (username, email, password_hash, full_name) 
         VALUES ($1, $2, $3, $4) 
         RETURNING id, username, email, password_hash, full_name, 
                   created_at, updated_at, is_active"
    )
    .bind(&user_data.username)
    .bind(&user_data.email)
    .bind(&user_data.password_hash)
    .bind(&user_data.full_name)
    .fetch_one(&mut **tx)
    .await?;
    
    Ok(User {
        id: row.get("id"),
        username: row.get("username"),
        email: row.get("email"),
        password_hash: row.get("password_hash"),
        full_name: row.get("full_name"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        is_active: row.get("is_active"),
    })
}
```

#### 6.2 连接池配置
```rust
use sqlx::postgres::PgPoolOptions;

pub async fn create_connection_pool(database_url: &str) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)           // 最大连接数
        .min_connections(5)            // 最小连接数
        .acquire_timeout(Duration::from_secs(30))  // 获取连接超时
        .idle_timeout(Duration::from_secs(600))    // 空闲连接超时
        .max_lifetime(Duration::from_secs(1800))   // 连接最大生命周期
        .connect(database_url)
        .await?;
    
    Ok(pool)
}
```

### 7. 测试和验证

#### 7.1 运行项目
```bash
# 编译
cargo build

# 运行
cargo run

# 或者使用脚本
./run.sh
```

#### 7.2 验证数据库
```bash
# 使用 sqlx-cli 验证数据库
sqlx migrate info

# 连接到数据库
psql postgresql://用户名:密码@主机:端口/数据库名

# 查看迁移表
SELECT * FROM _sqlx_migrations ORDER BY version;

# 查看创建的表
\dt

# 查看表结构
\d users
```

## 🎯 关键优势

1. **编译时 SQL 检查** - 在编译时验证 SQL 语法
2. **类型安全** - 编译时类型检查，避免运行时错误
3. **异步支持** - 基于 Tokio 的高性能异步操作
4. **迁移管理** - 内置数据库版本控制
5. **连接池** - 高效的数据库连接管理
6. **事务支持** - 完整的事务处理能力
7. **多数据库支持** - PostgreSQL、MySQL、SQLite、SQL Server

## 🚨 注意事项

1. **迁移文件命名** - 必须严格按照版本号_描述.sql 格式
2. **环境变量** - 确保 DATABASE_URL 正确配置
3. **依赖特性** - 根据数据库类型选择正确的 features
4. **错误处理** - 使用 anyhow 或 thiserror 进行错误处理
5. **日志记录** - 使用 tracing 进行结构化日志记录

## 🛠️ sqlx-cli 开发工具使用指南

### 安装和配置
```bash
# 安装 sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# 验证安装
sqlx --version

# 查看帮助
sqlx --help
```

### 数据库管理
```bash
# 创建数据库
sqlx database create

# 删除数据库（谨慎使用）
sqlx database drop

# 重置数据库（删除所有表和数据）
sqlx database reset

# 设置数据库 URL
export DATABASE_URL="postgresql://用户名:密码@主机:端口/数据库名"
```

### 迁移管理
```bash
# 创建新迁移
sqlx migrate add <迁移名称>

# 示例：
sqlx migrate add create_users_table
sqlx migrate add add_user_email_index
sqlx migrate add modify_user_status

# 运行迁移
sqlx migrate run

# 回滚迁移
sqlx migrate revert

# 查看迁移状态
sqlx migrate info

# 查看迁移历史
sqlx migrate list
```

### 开发辅助功能
```bash
# 验证 SQL 查询（编译时检查）
sqlx prepare

# 生成查询检查文件
sqlx prepare --check

# 离线模式运行（不需要数据库连接）
sqlx migrate run --offline

# 查看数据库连接状态
sqlx migrate info --connect-timeout 5
```

### 常用工作流程
```bash
# 1. 开发新功能时
sqlx migrate add add_new_feature
# 编辑生成的 .sql 文件
sqlx migrate run

# 2. 测试迁移
sqlx migrate run --offline  # 检查语法
sqlx migrate run            # 实际执行

# 3. 回滚测试
sqlx migrate revert         # 回滚最后一个迁移
sqlx migrate run            # 重新应用

# 4. 重置开发环境
sqlx database reset         # 清空数据库
sqlx migrate run            # 重新应用所有迁移
```

### 环境变量配置
```bash
# 在 .env 文件中设置
DATABASE_URL=postgresql://用户名:密码@主机:端口/数据库名
SQLX_OFFLINE=false          # 是否启用离线模式
SQLX_LOG=debug             # 日志级别
```

## 📚 参考资源

- [SQLx 官方文档](https://docs.rs/sqlx)
- [SQLx GitHub 仓库](https://github.com/launchbadge/sqlx)
- [SQLx CLI 文档](https://github.com/launchbadge/sqlx/tree/main/sqlx-cli)
- [PostgreSQL 官方文档](https://www.postgresql.org/docs/)
- [Rust 异步编程指南](https://rust-lang.github.io/async-book/)

