# Diesel 项目集成完整指南

## 🚀 概述

Diesel 是一个 Rust 的 ORM（对象关系映射）工具，提供类型安全的数据库查询、编译时 SQL 检查、自动迁移管理等功能。它支持 PostgreSQL、MySQL 和 SQLite 数据库。

## 📋 在新项目中引入 Diesel 的完整步骤

### 1. 项目初始化

#### 1.1 创建新项目
```bash
cargo new my_diesel_project
cd my_diesel_project
```

#### 1.2 配置 Cargo.toml
```toml
[package]
name = "my_diesel_project"
version = "0.1.0"
edition = "2021"

[dependencies]
# Diesel 核心依赖
diesel = { version = "2.2", features = [
    "postgres",      # PostgreSQL 支持
    "chrono",        # 时间类型支持
    "r2d2",          # 连接池支持
    "serde_json",    # JSON 类型支持
    "numeric"        # 精确数值类型支持
]}
diesel_migrations = "2.2"  # 迁移管理

# 异步运行时
tokio = { version = "1.0", features = ["full"] }

# 连接池
r2d2 = "0.8"

# 环境变量
dotenv = "0.15"
```

#### 1.3 安装 Diesel CLI
```bash
# 安装 Diesel CLI 工具
cargo install diesel_cli --no-default-features --features postgres

# 验证安装
diesel --version
```

### 2. 环境配置

#### 2.1 创建 .env 文件
```bash
# 数据库连接配置
DATABASE_URL=postgresql://用户名:密码@主机:端口/数据库名

# 示例：
DATABASE_URL=postgresql://sxt:default@localhost:5432/postgres1

# 日志级别
RUST_LOG=info
```

#### 2.2 配置 diesel.toml
```toml
# diesel.toml
[print_schema]
file = "src/schema.rs"
custom_type_derives = ["diesel::query_builder::QueryId"]

[migrations_directory]
dir = "migrations"
```

### 3. 数据库连接配置

#### 3.1 数据库管理器
```rust
// src/database.rs
use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::info;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

// 嵌入迁移文件
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// 数据库管理器
pub struct DatabaseManager {
    pub pool: DbPool,
}

impl DatabaseManager {
    /// 创建数据库连接池
    pub fn new() -> Result<Self> {
        info!("正在使用 Diesel 连接数据库...");
        
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://sxt:default@localhost:5432/postgres1".to_string());
            
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .context("无法创建数据库连接池")?;
        
        // 测试连接
        let _conn = pool.get().context("无法获取数据库连接")?;
        info!("数据库连接验证成功");
        
        Ok(Self { pool })
    }

    /// 执行数据库迁移
    pub fn safe_migrate(&self) -> Result<()> {
        info!("开始执行数据库迁移...");
        
        let mut conn = self.pool.get().context("无法获取数据库连接")?;
        
        match conn.run_pending_migrations(MIGRATIONS) {
            Ok(migrations) => {
                info!("✅ 数据库迁移完成，执行了 {} 个迁移", migrations.len());
                for migration in migrations {
                    info!("  - {}", migration);
                }
                Ok(())
            }
            Err(e) => {
                error!("❌ 迁移失败: {}", e);
                Err(anyhow::anyhow!("数据库迁移失败: {}", e))
            }
        }
    }
}

/// 便捷连接函数
pub fn connect() -> Result<DatabaseManager> {
    dotenv::dotenv().ok();
    DatabaseManager::new()
}
```

### 4. 数据模型定义

#### 4.1 定义模型结构

**重要说明：Diesel 会自动生成数据库表 schema！**

当你运行 `diesel migration run` 后，Diesel CLI 会自动：
1. 分析你的迁移文件（SQL 文件）
2. 生成对应的 Rust 代码到 `src/schema.rs` 文件中
3. 这个文件包含了所有表、列、索引、约束等的 Rust 表示

**注意：永远不要手动编辑 `src/schema.rs` 文件，它是由 Diesel CLI 自动生成的！**
```rust
// src/models.rs
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use bigdecimal::BigDecimal;

/// 用户模型
#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
    pub preferences: Option<serde_json::Value>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// 新用户插入模型
#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub is_active: Option<bool>,
    pub preferences: Option<serde_json::Value>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

/// 产品模型
#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::products)]
pub struct Product {
    pub id: i32,
    pub product_name: String,
    pub description: Option<String>,
    pub product_price: BigDecimal,
    pub category_id: Option<i32>,
    pub stock_quantity: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

/// 新产品插入模型
#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProduct {
    pub product_name: String,
    pub description: Option<String>,
    pub product_price: BigDecimal,
    pub category_id: Option<i32>,
    pub stock_quantity: Option<i32>,
    pub is_active: Option<bool>,
}

/// 订单模型
#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, Debug)]
#[diesel(belongs_to(User))]
#[diesel(table_name = crate::schema::orders)]
pub struct Order {
    pub id: i32,
    pub user_id: i32,
    pub order_number: String,
    pub total_amount: BigDecimal,
    pub status: Option<String>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// 新订单插入模型
#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::orders)]
pub struct NewOrder {
    pub user_id: i32,
    pub order_number: String,
    pub total_amount: BigDecimal,
    pub status: Option<String>,
    pub shipping_address: Option<String>,
    pub billing_address: Option<String>,
}
```

#### 4.2 Schema 自动生成说明

**Schema 文件的作用：**
- `src/schema.rs` 是 Diesel 的核心，定义了数据库结构
- 包含所有表的 `table!` 宏定义
- 定义了列名、类型、约束等元数据
- 为模型提供类型安全的表引用

**Schema 生成流程：**
```bash
# 1. 创建迁移文件
diesel migration generate create_users_table

# 2. 编辑迁移文件（up.sql 和 down.sql）

# 3. 运行迁移
diesel migration run

# 4. Diesel 自动生成 schema.rs
# 5. 在模型中使用 schema 引用表
```

**在模型中使用 schema：**
```rust
// 使用 schema 中定义的表名
#[diesel(table_name = crate::schema::users)]
pub struct User { ... }

// 或者直接使用
use crate::schema::users;
let result = users::table.load::<User>(&mut conn)?;
```

**Schema 文件示例结构：**
```rust
// src/schema.rs (自动生成，不要手动编辑)
table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        full_name -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        is_active -> Nullable<Bool>,
        preferences -> Nullable<Jsonb>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
    }
}

// 其他表的定义...
```

### 5. 数据库迁移系统

#### 5.1 创建迁移
```bash
# 创建新的迁移
diesel migration generate create_users_table

# 这会创建两个文件：
# migrations/YYYY-MM-DD-HHMMSS_create_users_table/up.sql
# migrations/YYYY-MM-DD-HHMMSS_create_users_table/down.sql
```

#### 5.2 迁移文件示例
```sql
-- up.sql - 创建用户表
CREATE TABLE users (
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
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);

-- down.sql - 回滚操作
DROP TABLE users;
```

#### 5.3 运行迁移
```bash
# 运行所有待处理的迁移
diesel migration run

# 回滚最后一个迁移
diesel migration revert

# 查看迁移状态
diesel migration list
```

### 6. 数据操作示例

#### 6.1 基础 CRUD 操作
```rust
// src/data.rs
use anyhow::Result;
use diesel::prelude::*;
use crate::database::DbPool;
use crate::models::*;
use crate::schema::*;
use tracing::info;

/// 查询所有用户
pub fn get_all_users(pool: &DbPool) -> Result<Vec<User>> {
    let mut conn = pool.get()?;
    
    let users = users::table
        .filter(users::is_active.eq(true))
        .order(users::created_at.desc())
        .load::<User>(&mut conn)?;
    
    info!("查询到 {} 个用户", users.len());
    Ok(users)
}

/// 根据ID查询用户
pub fn get_user_by_id(pool: &DbPool, user_id: i32) -> Result<Option<User>> {
    let mut conn = pool.get()?;
    
    let user = users::table
        .filter(users::id.eq(user_id))
        .filter(users::is_active.eq(true))
        .first::<User>(&mut conn)
        .optional()?;
    
    Ok(user)
}

/// 创建新用户
pub fn create_user(pool: &DbPool, new_user: &NewUser) -> Result<User> {
    let mut conn = pool.get()?;
    
    let user = diesel::insert_into(users::table)
        .values(new_user)
        .get_result::<User>(&mut conn)?;
    
    info!("用户创建成功: {}", user.username);
    Ok(user)
}

/// 更新用户信息
pub fn update_user(pool: &DbPool, user_id: i32, updates: &UserUpdate) -> Result<Option<User>> {
    let mut conn = pool.get()?;
    
    let updated_user = diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .filter(users::is_active.eq(true))
        .set((
            users::updated_at.eq(chrono::Utc::now()),
            updates.username.as_ref().map(|u| users::username.eq(u)),
            updates.email.as_ref().map(|e| users::email.eq(e)),
            updates.full_name.as_ref().map(|f| users::full_name.eq(f)),
        ))
        .get_result::<User>(&mut conn)
        .optional()?;
    
    Ok(updated_user)
}

/// 删除用户（软删除）
pub fn delete_user(pool: &DbPool, user_id: i32) -> Result<bool> {
    let mut conn = pool.get()?;
    
    let affected_rows = diesel::update(users::table)
        .filter(users::id.eq(user_id))
        .filter(users::is_active.eq(true))
        .set((
            users::is_active.eq(false),
            users::updated_at.eq(chrono::Utc::now()),
        ))
        .execute(&mut conn)?;
    
    Ok(affected_rows > 0)
}
```

#### 6.2 复杂查询示例
```rust
/// 分页查询用户
pub fn get_users_paginated(
    pool: &DbPool, 
    page: i64, 
    page_size: i64,
    search: Option<&str>
) -> Result<(Vec<User>, i64)> {
    let mut conn = pool.get()?;
    let offset = (page - 1) * page_size;
    
    // 构建查询
    let mut query = users::table.into_boxed();
    
    // 添加搜索条件
    if let Some(search_term) = search {
        let search_pattern = format!("%{}%", search_term);
        query = query.filter(
            users::username.ilike(&search_pattern)
                .or(users::email.ilike(&search_pattern))
                .or(users::full_name.ilike(&search_pattern))
        );
    }
    
    // 只查询活跃用户
    query = query.filter(users::is_active.eq(true));
    
    // 获取总数
    let total_count: i64 = query.clone().count().get_result(&mut conn)?;
    
    // 获取分页数据
    let users = query
        .order(users::created_at.desc())
        .offset(offset)
        .limit(page_size)
        .load::<User>(&mut conn)?;
    
    Ok((users, total_count))
}

/// 关联查询：获取用户及其订单
pub fn get_user_with_orders(pool: &DbPool, user_id: i32) -> Result<Option<(User, Vec<Order>)>> {
    let mut conn = pool.get()?;
    
    // 查询用户
    let user = users::table
        .filter(users::id.eq(user_id))
        .filter(users::is_active.eq(true))
        .first::<User>(&mut conn)
        .optional()?;
    
    match user {
        Some(user) => {
            // 查询用户的订单
            let orders = Order::belonging_to(&user)
                .order(orders::created_at.desc())
                .load::<Order>(&mut conn)?;
            
            Ok(Some((user, orders)))
        }
        None => Ok(None),
    }
}

/// 统计查询
pub fn get_user_statistics(pool: &DbPool) -> Result<UserStatistics> {
    let mut conn = pool.get()?;
    
    // 使用原始SQL进行复杂统计
    let stats: Vec<StatResult> = diesel::sql_query(
        "SELECT 
            COUNT(*) as total_users,
            COUNT(CASE WHEN created_at >= CURRENT_DATE - INTERVAL '30 days' THEN 1 END) as new_users_30d,
            COUNT(CASE WHEN created_at >= CURRENT_DATE - INTERVAL '7 days' THEN 1 END) as new_users_7d,
            COUNT(CASE WHEN created_at >= CURRENT_DATE THEN 1 END) as new_users_today
         FROM users 
         WHERE is_active = true"
    ).load(&mut conn)?;
    
    let stats = stats.first().unwrap();
    
    Ok(UserStatistics {
        total_users: stats.total_users,
        new_users_30d: stats.new_users_30d,
        new_users_7d: stats.new_users_7d,
        new_users_today: stats.new_users_today,
    })
}

#[derive(QueryableByName, Debug)]
pub struct StatResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub total_users: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub new_users_30d: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub new_users_7d: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub new_users_today: i64,
}
```

### 7. 主程序集成

#### 7.1 主程序示例
```rust
// src/main.rs
mod database;
mod models;
mod schema;
mod data;

use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 启动 Diesel 项目");
    
    // 加载环境变量
    dotenv::dotenv().ok();
    
    // 连接数据库
    let db_manager = match database::connect() {
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
    match db_manager.safe_migrate() {
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
    let new_user = NewUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        full_name: "测试用户".to_string(),
        is_active: Some(true),
        preferences: Some(serde_json::json!({
            "theme": "light",
            "language": "zh-CN"
        })),
        first_name: Some("测试".to_string()),
        last_name: Some("用户".to_string()),
    };
    
    match data::create_user(&db_manager.pool, &new_user) {
        Ok(user) => {
            info!("✅ 用户创建成功: {}", user.username);
        }
        Err(e) => {
            error!("❌ 用户创建失败: {}", e);
        }
    }
    
    // 示例：查询用户
    info!("🔍 查询用户列表...");
    match data::get_all_users(&db_manager.pool) {
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
    match data::get_users_paginated(&db_manager.pool, 1, 10, None) {
        Ok((users, total)) => {
            info!("✅ 分页查询成功: 第1页，共{}个用户，总数{}", users.len(), total);
        }
        Err(e) => {
            error!("❌ 分页查询失败: {}", e);
        }
    }
    
    info!("🎉 程序执行完成");
    
    Ok(())
}
```

### 8. 高级特性

#### 8.1 事务处理
```rust
/// 使用事务创建用户和订单
pub fn create_user_with_order(
    pool: &DbPool,
    user_data: &NewUser,
    order_data: &NewOrder
) -> Result<(User, Order)> {
    let mut conn = pool.get()?;
    
    // 开始事务
    conn.transaction(|conn| {
        // 在事务中创建用户
        let user = diesel::insert_into(users::table)
            .values(user_data)
            .get_result::<User>(conn)?;
        
        // 在事务中创建订单
        let order = diesel::insert_into(orders::table)
            .values(order_data)
            .get_result::<Order>(conn)?;
        
        Ok((user, order))
    })
}
```

#### 8.2 连接池配置
```rust
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub fn create_connection_pool(database_url: &str) -> Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    let pool = Pool::builder()
        .max_size(20)                    // 最大连接数
        .min_idle(Some(5))              // 最小空闲连接数
        .connection_timeout(Duration::from_secs(30))  // 连接超时
        .idle_timeout(Duration::from_secs(600))       // 空闲超时
        .max_lifetime(Some(Duration::from_secs(1800))) // 连接最大生命周期
        .build(manager)
        .context("无法创建连接池")?;
    
    Ok(pool)
}
```

### 9. 测试和验证

#### 9.1 运行项目
```bash
# 编译
cargo build

# 运行
cargo run

# 或者使用脚本
./run_diesel.sh
```

#### 9.2 验证数据库
```bash
# 连接到数据库
psql postgresql://用户名:密码@主机:端口/数据库名

# 查看迁移表
SELECT * FROM __diesel_schema_migrations ORDER BY version;

# 查看创建的表
\dt

# 查看表结构
\d users
```

#### 9.3 使用 Diesel CLI
```bash
# 创建新迁移
diesel migration generate add_user_roles

# 运行迁移
diesel migration run

# 回滚迁移
diesel migration revert

# 查看迁移状态
diesel migration list

# 重置数据库（危险操作）
diesel database reset
```

## 🎯 关键优势

1. **编译时 SQL 检查** - 在编译时验证 SQL 语法和类型
2. **类型安全** - 强类型系统，避免运行时错误
3. **自动迁移管理** - 内置数据库版本控制
4. **连接池支持** - 高效的数据库连接管理
5. **事务支持** - 完整的事务处理能力
6. **关联查询** - 支持复杂的表关系查询
7. **原始 SQL 支持** - 可以混合使用 ORM 和原始 SQL

## 🚨 注意事项

1. **迁移文件命名** - 使用 `diesel migration generate` 命令创建
2. **环境变量** - 确保 `DATABASE_URL` 正确配置
3. **依赖特性** - 根据数据库类型选择正确的 features
4. **Schema 生成** - 运行迁移后需要重新生成 schema.rs
5. **类型映射** - 注意 Rust 类型与数据库类型的对应关系

## 📚 参考资源

- [Diesel 官方文档](https://diesel.rs/)
- [Diesel 指南](https://diesel.rs/guides/)
- [Diesel GitHub 仓库](https://github.com/diesel-rs/diesel)
- [PostgreSQL 官方文档](https://www.postgresql.org/docs/)
- [Rust 异步编程指南](https://rust-lang.github.io/async-book/)

## 🔄 与 SQLx 的区别

| 特性 | Diesel | SQLx |
|------|--------|------|
| 类型 | ORM | SQL 工具包 |
| 查询方式 | 类型安全的查询构建器 | 原始 SQL + 宏 |
| 迁移管理 | 内置迁移系统 | 内置迁移系统 |
| 编译时检查 | 强类型检查 | SQL 语法检查 |
| 学习曲线 | 较陡峭 | 相对平缓 |
| 灵活性 | 结构化查询 | 高度灵活 |
| 性能 | 零成本抽象 | 零成本抽象 |

Diesel 适合需要强类型安全和结构化查询的项目，而 SQLx 适合需要高度灵活性和直接 SQL 控制的项目。

