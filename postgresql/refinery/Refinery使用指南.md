# Refinery 数据库迁移管理系统使用指南

## 📖 目录

- [概述](#概述)
- [工具安装](#工具安装)
- [数据库连接配置](#数据库连接配置)
- [迁移文件管理](#迁移文件管理)
- [开发环境：使用 CLI 管理迁移](#开发环境使用-cli-管理迁移)
- [生产环境：使用 Rust 代码执行迁移](#生产环境使用-rust-代码执行迁移)
- [迁移状态监控](#迁移状态监控)
- [错误处理和故障排除](#错误处理和故障排除)
- [最佳实践](#最佳实践)

## 📋 概述

Refinery 是专为 Rust 生态系统设计的数据库迁移工具，支持 PostgreSQL、MySQL和SQLite。本指南基于以下环境假设：

- ✅ **已安装 Rust** (rustc 1.70+)
- ✅ **已安装 PostgreSQL** (10+)
- ✅ **数据库服务正在运行**

### 🎯 使用场景

- **开发环境**: 使用 `refinery_cli` 命令行工具进行迁移开发和测试
- **生产环境**: 使用 `cargo run` 执行嵌入式迁移，确保应用启动时数据库结构正确

## 🔧 工具安装

### 安装 Refinery CLI（开发必备）

```bash
# 安装命令行工具
cargo install refinery_cli

# 验证安装
refinery --version
# 输出: refinery_cli 0.8.16
```

### 项目依赖配置

在 `Cargo.toml` 中添加 Refinery 相关依赖：

```toml
[dependencies]
# 核心迁移库
refinery = { version = "0.8", features = ["postgres"] }

# 数据库连接（选择一个）
tokio-postgres = { version = "0.7", features = ["with-chrono-0_4"] }  # 异步
postgres = { version = "0.19", features = ["with-chrono-0_4"] }       # 同步

# 辅助库
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenv = "0.15"
```

## 🔌 数据库连接配置

### 方式一：使用配置文件（推荐开发环境）

```bash
# 在项目根目录执行
refinery setup
```

交互式配置：
```
Select database 1) Mysql 2) Postgresql 3) Sqlite 4) Mssql: 2
Enter database host: localhost
Enter database port: 5432
Enter database username: sxt
Enter database password: default
Enter database name: postgres
```

生成的 `refinery.toml` 配置文件：
```toml
[main]
db_type = "Postgres"
db_host = "localhost"
db_port = "5432"
db_user = "sxt"
db_pass = "default"
db_name = "postgres"
trust_cert = false
```

### 方式二：使用环境变量（推荐生产环境）

创建 `.env` 文件：
```env
DATABASE_URL=postgresql://username:password@localhost:5432/database_name
RUST_LOG=info
```

**安全提示**: 生产环境中应通过系统环境变量或安全的密钥管理服务提供数据库凭据，避免在代码中硬编码敏感信息。

## 📁 迁移文件管理

### 命名规范（重要！）

Refinery 使用严格的文件命名格式：

```
V{版本号}__{描述}.sql
```

**规则说明**：
- `V` - 必须大写，表示版本化迁移  
- `{版本号}` - 数字版本号（推荐使用3位数字如 001, 002 以便排序）
- `__` - 双下划线分隔符
- `{描述}` - 英文描述，使用下划线连接单词(也可以使用中文)
- `.sql` - 文件扩展名

**✅ 正确示例**：
```
migrations/
├── V001__initial_schema.sql
├── V002__add_user_preferences.sql
├── V003__rename_product_column.sql
├── V004__create_indexes.sql
├── V005__add_constraints.sql
└── V010__major_refactor.sql
```

### 迁移文件编写规范

#### 基本结构模板

```sql
-- V001__initial_schema.sql
-- 描述：创建用户和产品相关的基础表结构
-- 作者：开发者姓名
-- 日期：2024-01-01

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
CREATE INDEX IF NOT EXISTS idx_users_active ON users(is_active);

-- 插入初始管理员用户（仅开发环境）
INSERT INTO users (username, email, password_hash, full_name) 
VALUES ('admin', 'admin@example.com', 'hashed_password', 'System Administrator')
ON CONFLICT (username) DO NOTHING;
```

#### 常见迁移类型示例

**1. 添加新表**
```sql
-- V002__create_orders_table.sql
CREATE TABLE IF NOT EXISTS orders (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    order_number VARCHAR(50) UNIQUE NOT NULL,
    total_amount DECIMAL(10,2) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

**2. 修改表结构**
```sql
-- V003__add_user_avatar.sql
-- 为用户表添加头像URL字段

ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(500);
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_login TIMESTAMPTZ;

-- 更新现有用户的默认头像
UPDATE users 
SET avatar_url = '/default-avatar.png' 
WHERE avatar_url IS NULL;
```

**3. 重命名列**
```sql
-- V004__rename_user_columns.sql
-- 重命名用户表中的列名以符合新的命名规范

ALTER TABLE users RENAME COLUMN full_name TO display_name;
ALTER TABLE users RENAME COLUMN is_active TO enabled;
```

**4. 创建索引**
```sql
-- V005__create_performance_indexes.sql
-- 为提高查询性能创建复合索引

CREATE INDEX IF NOT EXISTS idx_orders_user_status 
ON orders(user_id, status);

CREATE INDEX IF NOT EXISTS idx_orders_created_date 
ON orders(created_at DESC);

-- 条件索引（部分索引）
CREATE INDEX IF NOT EXISTS idx_active_users_email 
ON users(email) WHERE enabled = true;
```

**5. 数据迁移**
```sql
-- V006__migrate_user_status.sql
-- 将用户状态从布尔值迁移到枚举值

-- 添加新的状态列
ALTER TABLE users ADD COLUMN status VARCHAR(20) DEFAULT 'active';

-- 迁移现有数据
UPDATE users SET status = CASE 
    WHEN enabled = true THEN 'active'
    WHEN enabled = false THEN 'inactive'
    ELSE 'pending'
END;

-- 创建检查约束
ALTER TABLE users ADD CONSTRAINT chk_user_status 
CHECK (status IN ('active', 'inactive', 'pending', 'suspended'));

-- 删除旧列（可选，建议在后续迁移中进行）
-- ALTER TABLE users DROP COLUMN enabled;
```

**6. 删除数据（谨慎操作）**
```sql
-- V007__cleanup_test_data.sql
-- 清理测试环境的临时数据

-- 仅在非生产环境执行
DO $$
BEGIN
    -- 检查是否为生产环境（通过数据库名或特定标识）
    IF current_database() != 'production_db' THEN
        DELETE FROM orders WHERE order_number LIKE 'TEST_%';
        DELETE FROM users WHERE username LIKE 'test_%';
    END IF;
END $$;
```

## 🛠️ 开发环境：使用 CLI 管理迁移

在开发阶段，推荐使用 `refinery_cli` 进行迁移管理，它提供了灵活的测试和调试功能。

### 基本操作流程

#### 1. 创建新的迁移文件

```bash
# 在 migrations/ 目录下创建新文件
touch migrations/V008__add_user_profile.sql

# 编辑文件内容
vim migrations/V008__add_user_profile.sql
```

#### 2. 预览迁移（干运行）

```bash
# 查看将要执行的迁移，不实际执行
refinery migrate -f

# 输出示例：
# current version: 7
# applying migration: V8__add_user_profile
# not going to apply any migration as fake flag is enabled
```

#### 3. 执行迁移

```bash
# 执行所有待执行的迁移
refinery migrate

# 输出示例：
# applying migration: V8__add_user_profile
# migration V8__add_user_profile applied successfully
```

#### 4. 指定目标版本

```bash
# 迁移到指定版本
refinery migrate -t 5

# 回退到早期版本（注意：这不会撤销已执行的SQL）
refinery migrate -t 3 -f  # 先预览
refinery migrate -t 3     # 实际执行
```

### 开发环境常用命令

```bash
# 基础命令
refinery migrate                    # 执行所有待执行的迁移
refinery migrate -f                 # 预览模式，不实际执行
refinery migrate -c custom.toml     # 指定配置文件
refinery migrate -p ./sql/          # 指定迁移文件目录

# 严格模式（推荐）
refinery migrate -d                 # 发现分歧迁移时中止
refinery migrate -m                 # 发现缺失迁移时中止
refinery migrate -d -m              # 同时启用两种严格检查

# 事务模式
refinery migrate -g                 # 在单个事务中执行所有迁移
refinery migrate -g -f              # 预览事务性迁移

# 组合使用（推荐的开发工作流）
refinery migrate -f -d -m           # 预览 + 严格检查
refinery migrate -g -d -m           # 执行 + 事务 + 严格检查
```

### 开发工作流示例

```bash
# 1. 开发新功能前，确保数据库是最新的
refinery migrate -f -d -m
refinery migrate -g -d -m

# 2. 创建新的迁移文件
touch migrations/V009__add_product_categories.sql

# 3. 编写迁移SQL后，先预览
refinery migrate -f

# 4. 确认无误后执行
refinery migrate -g

# 5. 验证迁移结果
psql -d your_database -c "SELECT * FROM refinery_schema_history ORDER BY version DESC LIMIT 5;"
```

## 🚀 生产环境：使用 Rust 代码执行迁移

生产环境推荐将迁移嵌入到应用程序中，通过 `cargo run` 执行，确保应用启动时数据库结构是最新的。

### 项目结构

```
your-project/
├── Cargo.toml
├── .env                    # 环境变量配置
├── src/
│   ├── main.rs            # 主程序
│   ├── database.rs        # 数据库管理器
│   ├── data.rs           # 数据操作（可选）
│   └── tables.rs         # 表验证（可选）
└── migrations/           # 迁移文件目录
    ├── V001__initial_schema.sql
    ├── V002__add_user_preferences.sql
    └── ...
```

### Cargo.toml 配置

```toml
[package]
name = "your_app"
version = "0.1.0"
edition = "2021"

[dependencies]
# 核心迁移库
refinery = { version = "0.8", features = ["postgres"] }

# 数据库连接（同步版本，适合迁移）
postgres = { version = "0.19", features = ["with-chrono-0_4"] }

# 应用运行时数据库连接（异步版本，适合业务逻辑）
tokio-postgres = { version = "0.7", features = ["with-chrono-0_4"] }
tokio = { version = "1.0", features = ["full"] }

# 辅助库
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
dotenv = "0.15"

[[bin]]
name = "main"
path = "src/main.rs"
```

### 数据库管理器实现 (src/database.rs)

```rust
use anyhow::{Context, Result};
use tokio_postgres::{Client, NoTls};
use tracing::{info, error};
use std::str::FromStr;

// 嵌入迁移文件
mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub struct DatabaseManager {
    pub client: Client,
}

impl DatabaseManager {
    /// 创建数据库连接
    pub async fn new_with_config(database_url: &str) -> Result<Self> {
        info!("正在连接数据库...");
        
        let (client, connection) = tokio_postgres::connect(database_url, NoTls)
            .await
            .context("无法连接到数据库")?;
        
        // 在后台运行连接处理
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

    /// 执行嵌入式迁移（生产环境核心功能）
    pub async fn safe_migrate(&mut self, database_url: &str) -> Result<()> {
        info!("开始执行数据库迁移...");
        
        // 使用 spawn_blocking 避免运行时冲突
        let database_url_owned = database_url.to_owned();
        let migration_result = tokio::task::spawn_blocking(move || -> Result<refinery::Report> {
            // 创建同步连接用于迁移
            let db_config = postgres::Config::from_str(&database_url_owned)
                .context("解析数据库URL失败")?;
            
            let mut postgres_client = db_config.connect(postgres::NoTls)
                .context("创建迁移专用连接失败")?;
            
            // 执行嵌入的迁移
            let report = embedded::migrations::runner().run(&mut postgres_client)
                .map_err(|e| anyhow::anyhow!("迁移执行失败: {}", e))?;
            
            Ok(report)
        }).await
        .context("迁移任务执行失败")?
        .context("迁移操作失败")?;
        
        // 输出迁移结果
        let applied_count = migration_result.applied_migrations().len();
        if applied_count > 0 {
            info!("✅ 数据库迁移完成，应用了 {} 个迁移:", applied_count);
            for migration in migration_result.applied_migrations() {
                info!("  ✅ V{}: {}", migration.version(), migration.name());
            }
        } else {
            info!("✅ 数据库已是最新版本，无需迁移");
        }
        
        Ok(())
    }

    /// 获取客户端引用（用于业务逻辑）
    pub fn get_client(&self) -> &Client {
        &self.client
    }
}

/// 便捷连接函数
pub async fn connect() -> Result<DatabaseManager> {
    dotenv::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .context("未找到 DATABASE_URL 环境变量")?;
    
    DatabaseManager::new_with_config(&database_url).await
}
```

### 主程序实现 (src/main.rs)

```rust
mod database;

use anyhow::Result;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    info!("🚀 启动应用程序");
    
    // 加载环境变量
    dotenv::dotenv().ok();
    
    // 获取数据库URL
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            error!("未设置 DATABASE_URL 环境变量");
            std::process::exit(1);
        });
    
    // 创建数据库管理器
    let mut db_manager = match database::connect().await {
        Ok(manager) => {
            info!("✅ 数据库连接成功");
            manager
        }
        Err(e) => {
            error!("❌ 数据库连接失败: {}", e);
            std::process::exit(1);
        }
    };
    
    // 执行数据库迁移
    info!("📋 检查数据库迁移...");
    match db_manager.safe_migrate(&database_url).await {
        Ok(_) => {
            info!("✅ 数据库迁移检查完成");
        }
        Err(e) => {
            error!("❌ 数据库迁移失败: {}", e);
            std::process::exit(1);
        }
    }
    
    // 应用程序主要逻辑
    info!("🎯 开始运行主要业务逻辑");
    
    // 这里添加你的应用程序逻辑
    run_application(&db_manager).await?;
    
    info!("🎉 应用程序正常结束");
    Ok(())
}

async fn run_application(_db_manager: &database::DatabaseManager) -> Result<()> {
    // 你的业务逻辑代码
    // 例如：启动 web 服务器、处理消息队列等
    
    info!("业务逻辑运行中...");
    
    // 示例：简单的健康检查
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    Ok(())
}
```

### 生产环境部署脚本

```bash
#!/bin/bash
# deploy.sh - 生产环境部署脚本

echo "🚀 开始生产环境部署"

# 设置环境变量
export RUST_LOG=info
export DATABASE_URL="postgresql://prod_user:prod_pass@db-server:5432/prod_db"

# 构建应用程序
echo "🔨 构建应用程序..."
cargo build --release

# 运行应用程序（包含自动迁移）
echo "🚀 启动应用程序..."
./target/release/main

# 或者作为系统服务运行
# systemctl start your-app
```

### 生产环境运行示例

```bash
# 设置环境变量
export DATABASE_URL="postgresql://username:password@localhost:5432/production_db"
export RUST_LOG=info

# 运行应用程序（自动执行迁移）
cargo run --bin main

# 输出示例：
# 2024-01-01T10:00:00Z  INFO main: 🚀 启动应用程序
# 2024-01-01T10:00:01Z  INFO database: 正在连接数据库...
# 2024-01-01T10:00:01Z  INFO database: 数据库连接验证成功
# 2024-01-01T10:00:01Z  INFO main: ✅ 数据库连接成功
# 2024-01-01T10:00:01Z  INFO main: 📋 检查数据库迁移...
# 2024-01-01T10:00:02Z  INFO database: 开始执行数据库迁移...
# 2024-01-01T10:00:02Z  INFO database: ✅ 数据库已是最新版本，无需迁移
# 2024-01-01T10:00:02Z  INFO main: ✅ 数据库迁移检查完成
# 2024-01-01T10:00:02Z  INFO main: 🎯 开始运行主要业务逻辑
```

## 📊 迁移状态监控

### Refinery Schema History 表结构

Refinery 自动创建 `refinery_schema_history` 表来跟踪迁移状态：

```sql
-- 查看表结构
\d refinery_schema_history;

-- 表结构说明
CREATE TABLE refinery_schema_history (
    version INTEGER PRIMARY KEY,         -- 迁移版本号
    name VARCHAR(255) NOT NULL,          -- 迁移名称（不含版本号）
    applied_on TIMESTAMP NOT NULL,       -- 应用时间
    checksum VARCHAR(255) NOT NULL       -- 迁移文件的校验和
);
```

**字段说明**：
- `version`: 迁移的版本号（如 1, 2, 3）
- `name`: 迁移的描述名称（如 "initial_schema", "add_user_preferences"）
- `applied_on`: 迁移被应用的时间戳
- `checksum`: 迁移文件内容的 MD5 校验和，用于检测文件是否被修改

### 查询迁移状态

```sql
-- 查看所有已应用的迁移
SELECT version, name, applied_on, checksum 
FROM refinery_schema_history 
ORDER BY version;

-- 查看最近5个迁移
SELECT version, name, applied_on 
FROM refinery_schema_history 
ORDER BY version DESC 
LIMIT 5;

-- 查看当前数据库版本
SELECT MAX(version) as current_version 
FROM refinery_schema_history;

-- 检查特定迁移是否已应用
SELECT EXISTS(
    SELECT 1 FROM refinery_schema_history 
    WHERE version = 5
) as migration_applied;
```

### 监控脚本示例

```bash
#!/bin/bash
# migration_status.sh - 迁移状态监控脚本

DB_URL="postgresql://username:password@localhost:5432/database"

echo "📊 数据库迁移状态报告"
echo "==============================="

# 查看当前版本
CURRENT_VERSION=$(psql "$DB_URL" -tAc "SELECT COALESCE(MAX(version), 0) FROM refinery_schema_history;")
echo "当前数据库版本: $CURRENT_VERSION"

# 查看迁移历史
echo ""
echo "迁移历史:"
psql "$DB_URL" -c "
SELECT 
    'V' || LPAD(version::text, 3, '0') as Version,
    name as Migration_Name,
    applied_on::date as Applied_Date,
    applied_on::time as Applied_Time
FROM refinery_schema_history 
ORDER BY version;
"

# 检查文件系统中的迁移文件数量
MIGRATION_FILES=$(ls -1 migrations/V*.sql 2>/dev/null | wc -l)
echo ""
echo "文件系统中的迁移数量: $MIGRATION_FILES"
echo "数据库中的迁移数量: $CURRENT_VERSION"

if [ "$MIGRATION_FILES" -gt "$CURRENT_VERSION" ]; then
    PENDING=$((MIGRATION_FILES - CURRENT_VERSION))
    echo "⚠️  有 $PENDING 个迁移待应用"
else
    echo "✅ 数据库是最新版本"
fi
```

## ⚠️ 错误处理和故障排除

### 常见错误类型及解决方案

#### 1. 连接相关错误

**错误示例**：
```
Error: unable to connect to database
Connection refused
```

**可能原因**：
- PostgreSQL 服务未启动
- 连接参数错误（主机、端口、用户名、密码）
- 网络防火墙阻止连接
- 数据库不存在

**解决方案**：
```bash
# 检查 PostgreSQL 服务状态
sudo systemctl status postgresql
# 或
brew services list | grep postgresql

# 启动 PostgreSQL 服务
sudo systemctl start postgresql
# 或
brew services start postgresql

# 测试连接
psql "postgresql://username:password@localhost:5432/database" -c "SELECT 1;"

# 检查数据库是否存在
psql "postgresql://username:password@localhost:5432/postgres" -c "\l"
```

#### 2. 权限相关错误

**错误示例**：
```
Error: permission denied for table refinery_schema_history
Error: permission denied for database
```

**解决方案**：
```sql
-- 作为超级用户执行以下命令
-- 授予数据库权限
GRANT ALL PRIVILEGES ON DATABASE your_database TO your_user;

-- 授予schema权限
GRANT ALL PRIVILEGES ON SCHEMA public TO your_user;

-- 授予表权限
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO your_user;

-- 授予序列权限
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO your_user;

-- 设置默认权限
ALTER DEFAULT PRIVILEGES IN SCHEMA public 
GRANT ALL PRIVILEGES ON TABLES TO your_user;

ALTER DEFAULT PRIVILEGES IN SCHEMA public 
GRANT ALL PRIVILEGES ON SEQUENCES TO your_user;
```

#### 3. 迁移文件相关错误

**错误A：校验和不匹配**
```
Error: checksum mismatch for migration V001__initial_schema
Expected: abc123, Found: def456
```

**原因**：迁移文件在应用后被修改

**解决方案**：
```bash
# 方案1：不要修改已应用的迁移文件，创建新的迁移文件
touch migrations/V009__fix_initial_schema.sql

# 方案2：如果确实需要修改（仅限开发环境）
# 备份数据
pg_dump your_database > backup.sql

# 删除迁移记录（危险操作！）
psql your_database -c "DELETE FROM refinery_schema_history WHERE version = 1;"

# 重新应用迁移
refinery migrate
```

**错误B：SQL语法错误**
```
Error: syntax error at or near "CREAT"
```

**解决方案**：
```bash
# 使用干运行模式预先检查
refinery migrate -f

# 使用 PostgreSQL 客户端测试SQL
psql your_database < migrations/V008__problematic_migration.sql

# 修复SQL语法后重新尝试
refinery migrate
```

#### 4. 运行时冲突错误（Rust集成）

**错误示例**：
```
Error: Cannot start a runtime from within a runtime
```

**解决方案**：
```rust
// 错误的做法 - 在异步函数中直接使用同步客户端
async fn wrong_migrate() {
    let mut client = postgres::Client::connect("...", postgres::NoTls)?;
    embedded::migrations::runner().run(&mut client)?; // 会导致运行时冲突
}

// 正确的做法 - 使用 spawn_blocking
async fn correct_migrate() -> Result<()> {
    let database_url = "postgresql://...".to_owned();
    
    let result = tokio::task::spawn_blocking(move || -> Result<refinery::Report> {
        let db_config = postgres::Config::from_str(&database_url)?;
        let mut client = db_config.connect(postgres::NoTls)?;
        let report = embedded::migrations::runner().run(&mut client)?;
        Ok(report)
    }).await??;
    
    Ok(())
}
```

#### 5. 事务相关错误

**错误示例**：
```
Error: current transaction is aborted, commands ignored until end of transaction block
```

**解决方案**：
```bash
# 使用事务模式执行迁移，确保原子性
refinery migrate -g

# 如果迁移失败，检查具体的SQL错误
# 修复后重新执行
```

### 迁移失败恢复策略

#### 开发环境恢复

```bash
#!/bin/bash
# dev_recovery.sh - 开发环境迁移失败恢复

echo "🔧 开发环境迁移失败恢复"

# 1. 备份当前数据
pg_dump your_database > "backup_$(date +%Y%m%d_%H%M%S).sql"

# 2. 检查迁移状态
echo "当前迁移状态:"
psql your_database -c "SELECT * FROM refinery_schema_history ORDER BY version DESC LIMIT 5;"

# 3. 如果需要，删除失败的迁移记录
read -p "是否删除最后一个迁移记录？(y/N): " confirm
if [ "$confirm" = "y" ]; then
    LAST_VERSION=$(psql your_database -tAc "SELECT MAX(version) FROM refinery_schema_history;")
    psql your_database -c "DELETE FROM refinery_schema_history WHERE version = $LAST_VERSION;"
    echo "已删除版本 $LAST_VERSION 的迁移记录"
fi

# 4. 重新尝试迁移
echo "重新执行迁移..."
refinery migrate -f  # 先预览
refinery migrate -g  # 再执行
```

#### 生产环境恢复（谨慎操作）

```bash
#!/bin/bash
# prod_recovery.sh - 生产环境迁移失败恢复

echo "🚨 生产环境迁移失败恢复"

# 1. 立即创建完整备份
echo "创建完整备份..."
pg_dump your_production_db > "critical_backup_$(date +%Y%m%d_%H%M%S).sql"

# 2. 检查迁移状态和错误
echo "检查迁移状态..."
psql your_production_db -c "
SELECT 
    version,
    name,
    applied_on,
    CASE 
        WHEN applied_on > NOW() - INTERVAL '1 hour' THEN '🔴 Recent'
        ELSE '✅ Old'
    END as status
FROM refinery_schema_history 
ORDER BY version DESC 
LIMIT 10;
"

# 3. 检查应用程序状态
echo "检查应用程序状态..."
# 这里添加你的健康检查逻辑

# 4. 如果需要回滚到备份
read -p "是否需要从备份恢复？(y/N): " confirm
if [ "$confirm" = "y" ]; then
    echo "⚠️  从备份恢复（这将丢失最近的数据更改）"
    # psql your_production_db < your_backup_file.sql
fi
```

## 🎯 最佳实践

### 开发工作流

#### 1. 迁移开发流程

```bash
# 步骤1: 创建新分支
git checkout -b feature/add-user-profiles

# 步骤2: 确保数据库是最新的
refinery migrate -f -d -m
refinery migrate -g -d -m

# 步骤3: 开发功能和创建迁移
touch migrations/V008__create_user_profiles.sql

# 步骤4: 编写迁移SQL
cat > migrations/V008__create_user_profiles.sql << 'EOF'
-- V008__create_user_profiles.sql
-- 创建用户档案扩展表

CREATE TABLE IF NOT EXISTS user_profiles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bio TEXT,
    website VARCHAR(500),
    location VARCHAR(200),
    birth_date DATE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_user_profiles_user_id ON user_profiles(user_id);
CREATE INDEX IF NOT EXISTS idx_user_profiles_location ON user_profiles(location);
EOF

# 步骤5: 测试迁移
refinery migrate -f  # 预览
refinery migrate -g  # 执行

# 步骤6: 验证结果
psql your_database -c "\d user_profiles"
psql your_database -c "SELECT * FROM refinery_schema_history WHERE version = 8;"

# 步骤7: 提交更改
git add migrations/V008__create_user_profiles.sql
git commit -m "feat: add user profiles table migration"
```

#### 2. 团队协作规范

**分支合并前检查**：
```bash
# 合并前检查脚本
#!/bin/bash
# pre_merge_check.sh

echo "🔍 合并前迁移检查"

# 1. 检查是否有新的迁移文件
NEW_MIGRATIONS=$(git diff main --name-only | grep "migrations/V.*\.sql" | wc -l)
echo "新增迁移文件数量: $NEW_MIGRATIONS"

# 2. 检查文件命名是否正确
for file in $(git diff main --name-only | grep "migrations/"); do
    if [[ ! "$file" =~ migrations/V[0-9]{3}__[a-z0-9_]+\.sql$ ]]; then
        echo "❌ 文件命名不规范: $file"
        exit 1
    fi
done

# 3. 检查版本号是否连续
LATEST_MAIN=$(git show main:migrations/ | grep -o "V[0-9]\{3\}" | sort -V | tail -1 | tr -d 'V')
LATEST_BRANCH=$(ls migrations/V*.sql | grep -o "V[0-9]\{3\}" | sort -V | tail -1 | tr -d 'V')

if [ $((LATEST_BRANCH)) -gt $((LATEST_MAIN + NEW_MIGRATIONS)) ]; then
    echo "❌ 版本号不连续，请重新调整版本号"
    exit 1
fi

echo "✅ 迁移文件检查通过"
```

**版本号管理**：
```bash
# 获取下一个可用版本号的脚本
#!/bin/bash
# next_version.sh

CURRENT_MAX=$(ls migrations/V*.sql 2>/dev/null | grep -o "V[0-9]\{3\}" | sort -V | tail -1 | tr -d 'V' | sed 's/^0*//')
NEXT_VERSION=$((CURRENT_MAX + 1))
PADDED_VERSION=$(printf "%03d" $NEXT_VERSION)

echo "下一个迁移版本号: V${PADDED_VERSION}"
echo "创建迁移文件: migrations/V${PADDED_VERSION}__your_description.sql"
```

### 生产环境最佳实践

#### 1. 部署前检查清单

- [ ] **备份验证**: 确保有完整的数据库备份
- [ ] **迁移测试**: 在与生产环境相同的数据副本上测试迁移
- [ ] **回滚计划**: 准备应急回滚方案
- [ ] **监控准备**: 设置迁移监控和告警
- [ ] **维护窗口**: 安排合适的维护时间窗口
- [ ] **团队通知**: 通知相关团队成员

#### 2. 零停机迁移策略

**兼容性迁移**：
```sql
-- V010__add_phone_column_compatible.sql
-- 第一阶段：添加可为空的列
ALTER TABLE users ADD COLUMN phone VARCHAR(20);

-- 第二阶段：在应用代码中开始使用新列
-- 第三阶段：在后续迁移中添加约束
```

```sql
-- V011__make_phone_required.sql
-- 当所有用户都有电话号码后，添加约束
UPDATE users SET phone = '未设置' WHERE phone IS NULL;
ALTER TABLE users ALTER COLUMN phone SET NOT NULL;
```

**大表变更策略**：
```sql
-- V012__migrate_large_table.sql
-- 对大表进行分批处理，避免长时间锁表

-- 创建新表
CREATE TABLE users_new (LIKE users INCLUDING ALL);

-- 分批迁移数据（通过应用程序执行）
-- INSERT INTO users_new SELECT * FROM users WHERE id BETWEEN ? AND ?;

-- 最后切换表名
-- ALTER TABLE users RENAME TO users_old;
-- ALTER TABLE users_new RENAME TO users;
```

### 性能和安全最佳实践

#### 1. 迁移性能优化

```sql
-- V013__performance_optimized_migration.sql
-- 性能优化的迁移示例

-- 1. 使用并发索引创建（不阻塞表）
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_orders_created_at ON orders(created_at);

-- 2. 避免全表扫描的更新
UPDATE users SET status = 'active' 
WHERE id IN (
    SELECT id FROM users 
    WHERE status IS NULL 
    LIMIT 1000
);

-- 3. 使用适当的工作内存设置
SET work_mem = '256MB';

-- 执行大型操作后恢复
RESET work_mem;
```

#### 2. 安全实践

```sql
-- V014__security_best_practices.sql
-- 安全最佳实践示例

-- 1. 添加行级安全策略前先创建策略
ALTER TABLE user_profiles ENABLE ROW LEVEL SECURITY;

CREATE POLICY user_profiles_policy ON user_profiles
    FOR ALL TO application_user
    USING (user_id = current_setting('app.current_user_id')::integer);

-- 2. 敏感数据加密
ALTER TABLE users ADD COLUMN encrypted_ssn BYTEA;

-- 3. 审计日志
CREATE TABLE IF NOT EXISTS audit_log (
    id SERIAL PRIMARY KEY,
    table_name VARCHAR(50) NOT NULL,
    operation VARCHAR(10) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    changed_by INTEGER,
    changed_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

### 代码质量保证

#### 1. 迁移代码审查检查项

- **文件命名**: 是否遵循 `V{版本}__{描述}.sql` 格式
- **SQL语法**: 是否使用了正确的PostgreSQL语法
- **安全性**: 是否使用了 `IF NOT EXISTS` 等防御性语句
- **性能影响**: 大表操作是否考虑了性能影响
- **回滚方案**: 是否有明确的回滚计划
- **测试覆盖**: 是否在测试环境验证过

#### 2. 自动化质量检查

```yaml
# .github/workflows/migration-check.yml
name: Migration Quality Check

on:
  pull_request:
    paths: ['migrations/**']

jobs:
  migration-check:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Check Migration Naming
      run: |
        for file in migrations/V*.sql; do
          if [[ ! "$file" =~ ^migrations/V[0-9]{3}__[a-z0-9_]+\.sql$ ]]; then
            echo "❌ Invalid naming: $file"
            exit 1
          fi
        done
    
    - name: Check SQL Syntax
      run: |
        # 使用 PostgreSQL 客户端检查语法
        for file in migrations/V*.sql; do
          psql -d postgres -f "$file" --dry-run || exit 1
        done
    
    - name: Test Migration
      run: |
        # 在测试数据库中执行迁移
        refinery migrate -f
        refinery migrate
```

---

## 📚 总结

本指南涵盖了 Refinery 数据库迁移管理的完整流程：

### 🎯 关键要点

1. **开发环境**：使用 `refinery_cli` 进行迁移开发和测试
2. **生产环境**：使用 `cargo run` 执行嵌入式迁移
3. **迁移文件**：严格遵循命名规范和编写最佳实践
4. **错误处理**：建立完善的错误处理和恢复机制
5. **团队协作**：制定清晰的工作流程和质量标准

### 🔗 参考资源

- [Refinery 官方文档](https://docs.rs/refinery/)
- [PostgreSQL 官方文档](https://www.postgresql.org/docs/)
- [数据库迁移最佳实践](https://martinfowler.com/articles/evodb.html)

**祝您使用愉快！** 🎉
