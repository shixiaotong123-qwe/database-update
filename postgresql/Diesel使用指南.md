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

# 错误处理
anyhow = "1.0"

# 日志
tracing = "0.1"
tracing-subscriber = "0.3"
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

### 3. 数据库初始化

#### 3.1 初始化 Diesel 项目
```bash
# 重要：这一步会创建必要的目录和文件
diesel setup

# 这会创建：
# - migrations/ 目录
# - diesel.toml 配置文件
# - 如果数据库不存在，会尝试创建数据库
```

### 4. 创建第一个迁移

#### 4.1 生成迁移文件
```bash
# 创建用户表迁移
diesel migration generate create_users_table

# 这会创建：
# migrations/YYYY-MM-DD-HHMMSS_create_users_table/up.sql
# migrations/YYYY-MM-DD-HHMMSS_create_users_table/down.sql
```

#### 4.2 编写迁移文件
```sql
-- migrations/YYYY-MM-DD-HHMMSS_create_users_table/up.sql
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

-- migrations/YYYY-MM-DD-HHMMSS_create_users_table/down.sql
DROP TABLE users;
```

#### 4.3 运行迁移
```bash
# 执行迁移
diesel migration run

# 这会：
# 1. 在数据库中执行 up.sql 中的 SQL 语句
# 2. 在 __diesel_schema_migrations 表中记录迁移历史
# 3. 但不会自动生成 schema.rs 文件！
```

### 5. 生成 Schema 文件（关键步骤！）

#### 5.1 手动生成 Schema
```bash
# 重要：运行迁移后，需要手动生成 schema.rs
diesel print-schema > src/schema.rs

# 这个命令会：
# 1. 连接到数据库
# 2. 读取当前的表结构
# 3. 生成对应的 Rust 代码到 schema.rs
```

#### 5.2 自动生成 Schema（推荐）
```toml
# 在 diesel.toml 中配置自动生成
[print_schema]
file = "src/schema.rs"
custom_type_derives = ["diesel::query_builder::QueryId"]

# 这样每次运行 diesel migration run 后会自动更新 schema.rs
```

#### 5.3 运行时自动生成 Schema（生产环境推荐）
```rust
// 在代码中集成自动 Schema 生成功能
// 这样运行 cargo run 时就能自动生成 Schema

// src/schema_generator.rs
use anyhow::{Context, Result};
use std::process::Command;
use tracing::{info, warn};

/// Schema 自动生成器
pub struct SchemaGenerator;

impl SchemaGenerator {
    /// 自动生成 Schema 文件
    pub fn generate_schema() -> Result<()> {
        info!("🔄 开始自动生成 Schema 文件...");
        
        // 检查 diesel CLI 是否可用
        if !Self::check_diesel_cli() {
            warn!("⚠️  Diesel CLI 不可用，跳过 Schema 生成");
            return Ok(());
        }
        
        // 执行 diesel print-schema 命令
        let output = Command::new("diesel")
            .args(&["print-schema"])
            .output()
            .context("无法执行 diesel CLI 命令")?;
        
        if output.status.success() {
            let schema_content = String::from_utf8(output.stdout)
                .context("无法解析 schema 内容")?;
            
            // 写入 schema.rs 文件
            std::fs::write("src/schema.rs", schema_content)
                .context("无法写入 schema.rs 文件")?;
            
            info!("✅ Schema 文件自动生成成功");
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            warn!("⚠️  Schema 自动生成失败: {}", error_msg);
            info!("💡 请手动运行: diesel print-schema > src/schema.rs");
        }
        
        Ok(())
    }
    
    /// 检查 diesel CLI 是否可用
    fn check_diesel_cli() -> bool {
        Command::new("diesel")
            .arg("--version")
            .output()
            .is_ok()
    }
}
```

**使用方法：**
```rust
// 在迁移执行后自动调用
pub fn safe_migrate(&self) -> Result<()> {
    // ... 执行迁移 ...
    
    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(migrations) => {
            if !migrations.is_empty() {
                // 自动生成 Schema
                self.auto_generate_schema()?;
            }
            Ok(())
        }
        // ... 错误处理
    }
}
```

#### 5.3 生成的 Schema 文件示例
```rust
// src/schema.rs (自动生成，不要手动编辑)
// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        email -> Varchar,
        password_hash -> Varchar,
        full_name -> Varchar,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        is_active -> Nullable<Bool>,
    }
}

// 允许表在同一查询中出现
diesel::allow_tables_to_appear_in_same_query!(
    users,
);
```

### 6. 定义数据模型

#### 6.1 创建模型文件
```rust
// src/models.rs
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 用户模型 - 用于查询
#[derive(Queryable, Identifiable, Serialize, Deserialize, Debug)]
#[diesel(table_name = crate::schema::users)]  // 关联到 schema 中定义的表
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub is_active: Option<bool>,
}

/// 新用户模型 - 用于插入
#[derive(Insertable, Debug)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub full_name: String,
    pub is_active: Option<bool>,
}
```

#### 6.2 模型与 Schema 的关系说明
```rust
// 重要理解：
// 1. schema.rs 定义了数据库表结构（由 Diesel CLI 生成）
// 2. models.rs 定义了 Rust 结构体（手动编写）
// 3. 通过 #[diesel(table_name = crate::schema::users)] 关联

// 这样 Diesel 就知道：
// - User 结构体对应 users 表
// - 字段类型与数据库列类型匹配
// - 提供类型安全的查询
```

### 7. 数据库连接配置

#### 7.1 数据库管理器
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
        
        // 首先检查并设置数据库基线
        self.check_and_setup_database()?;
        
        let mut conn = self.pool.get().context("无法获取数据库连接")?;
        
        match conn.run_pending_migrations(MIGRATIONS) {
            Ok(migrations) => {
                if migrations.is_empty() {
                    info!("✅ 数据库已是最新状态，无需执行迁移");
                } else {
                    info!("✅ 数据库迁移完成，执行了 {} 个迁移", migrations.len());
                    for migration in migrations {
                        info!("  - {}", migration);
                    }
                    
                    // 自动生成 Schema 文件
                    self.auto_generate_schema()?;
                }
                Ok(())
            }
            Err(e) => {
                error!("❌ 迁移失败: {}", e);
                Err(anyhow::anyhow!("数据库迁移失败: {}", e))
            }
        }
    }
    
    /// 自动生成 Schema 文件
    fn auto_generate_schema(&self) -> Result<()> {
        use crate::schema_generator::SchemaGenerator;
        SchemaGenerator::generate_schema()
    }
}

/// 便捷连接函数
pub fn connect() -> Result<DatabaseManager> {
    dotenv::dotenv().ok();
    DatabaseManager::new()
}
```

### 8. 数据操作示例

#### 8.1 基础 CRUD 操作
```rust
// src/data.rs
use anyhow::Result;
use diesel::prelude::*;
use crate::database::DbPool;
use crate::models::*;
use crate::schema::*;  // 导入 schema 中定义的表
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
```

#### 8.2 复杂查询示例
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
```

### 9. 主程序集成

#### 9.1 主程序示例
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
    
    info!("🎉 程序执行完成");
    
    Ok(())
}
```

### 10. 完整的项目工作流程

#### 10.1 开发流程
```bash
# 1. 项目初始化
cargo new my_diesel_project
cd my_diesel_project

# 2. 配置依赖
# 编辑 Cargo.toml

# 3. 安装 Diesel CLI
cargo install diesel_cli --no-default-features --features postgres

# 4. 环境配置
# 创建 .env 文件
# 配置 diesel.toml

# 5. 初始化项目
diesel setup

# 6. 创建迁移
diesel migration generate create_users_table

# 7. 编辑迁移文件
# 编写 up.sql 和 down.sql

# 8. 运行迁移
diesel migration run

# 9. 生成 Schema（关键步骤！）
diesel print-schema > src/schema.rs

# 10. 定义模型
# 在 models.rs 中编写结构体

# 11. 编写业务逻辑
# 在 data.rs 中编写数据操作函数

# 12. 集成到主程序
# 在 main.rs 中调用

# 13. 编译运行
cargo run
```

#### 10.1.1 自动 Schema 生成流程（推荐）
```bash
# 1-7. 同上

# 8. 运行迁移并自动生成 Schema
cargo run  # 自动执行迁移并生成 Schema

# 或者手动执行
diesel migration run
cargo run  # 自动生成 Schema
```

#### 10.2 后续开发流程
```bash
# 当需要修改数据库结构时：

# 1. 创建新迁移
diesel migration generate add_user_roles

# 2. 编辑迁移文件
# 编写 up.sql 和 down.sql

# 3. 运行迁移
diesel migration run

# 4. 重新生成 Schema
diesel print-schema > src/schema.rs

# 5. 更新模型（如果需要）
# 修改 models.rs

# 6. 编译运行
cargo run
```

#### 10.2.1 自动 Schema 生成流程（推荐）
```bash
# 1-2. 同上

# 3. 运行迁移并自动生成 Schema
cargo run  # 自动执行迁移并生成 Schema

# 4. 更新模型（如果需要）
# 修改 models.rs

# 5. 编译运行
cargo run
```

### 11. 高级特性

#### 11.1 事务处理
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

#### 11.2 连接池配置
```rust
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::time::Duration;

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

### 12. 测试和验证

#### 12.1 运行项目
```bash
# 编译
cargo build

# 运行
cargo run

# 或者使用脚本
./run_diesel.sh
```

#### 12.2 验证数据库
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

#### 12.3 使用 Diesel CLI
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

# 重新生成 Schema
diesel print-schema > src/schema.rs
```

## 🎯 关键优势

1. **编译时 SQL 检查** - 在编译时验证 SQL 语法和类型
2. **类型安全** - 强类型系统，避免运行时错误
3. **自动迁移管理** - 内置数据库版本控制
4. **连接池支持** - 高效的数据库连接管理
5. **事务支持** - 完整的事务处理能力
6. **关联查询** - 支持复杂的表关系查询
7. **原始 SQL 支持** - 可以混合使用 ORM 和原始 SQL
8. **自动 Schema 生成** - 运行时自动生成和更新 Schema 文件

## 🚨 重要注意事项

### 1. **Schema 文件管理**
- **永远不要手动编辑** `src/schema.rs` 文件
- 这个文件由 Diesel CLI 自动生成
- 每次修改数据库结构后都需要重新生成

### 2. **迁移文件命名**
- 使用 `diesel migration generate` 命令创建
- 不要手动创建或重命名迁移文件
- 迁移文件名会自动包含时间戳

### 3. **环境变量配置**
- 确保 `DATABASE_URL` 正确配置
- 在生产环境中使用环境变量，不要硬编码

### 4. **依赖特性选择**
- 根据数据库类型选择正确的 features
- PostgreSQL: `"postgres"`
- MySQL: `"mysql"`
- SQLite: `"sqlite"`

### 5. **类型映射**
- 注意 Rust 类型与数据库类型的对应关系
- 使用 `diesel::sql_types` 中的类型进行自定义映射

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

## 🎉 总结

Diesel 是一个功能强大的 Rust ORM，特别适合需要强类型安全和结构化查询的项目。通过遵循本指南的步骤，你可以成功集成 Diesel 到你的 Rust 项目中，享受类型安全的数据库操作体验。

**记住关键点：**
1. 运行迁移后必须生成 Schema 文件
2. 不要手动编辑自动生成的文件
3. 遵循正确的开发流程
4. 充分利用 Diesel 的类型安全特性
5. 使用自动 Schema 生成功能简化部署流程

## 🚀 自动 Schema 生成功能

### 功能特点
- **开发时**：可以使用 Diesel CLI 管理迁移
- **生产环境**：只需要运行 `cargo run` 就能自动执行迁移并生成 Schema
- **CI/CD**：集成到自动化部署流程中

### 使用方法
```bash
# 自动模式（推荐）
cargo run  # 自动执行迁移并生成 Schema

# 强制重新生成 Schema
FORCE_REGENERATE_SCHEMA=1 cargo run

# 开发时的完整流程
diesel migration generate xxx
# 编辑迁移文件
cargo run  # 自动执行迁移并生成 Schema
```

### 优势
1. **完全自动化**：无需手动执行 `diesel print-schema`
2. **生产就绪**：服务器上只需要 `cargo run`
3. **开发友好**：开发时仍可使用 CLI 工具
4. **错误处理**：优雅处理各种异常情况
5. **一致性保证**：Schema 始终与数据库结构同步
