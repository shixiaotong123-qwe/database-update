# ClickHouse 数据库连接器和迁移工具

## 🚀 概述

这是一个 ClickHouse 数据库连接器和迁移工具，采用连接管理器架构，避免重复创建数据库连接。

## 🏗️ 架构设计

### 连接管理器模式

为了避免重复创建 ClickHouse 连接，我们采用了连接管理器模式：

```rust
// 创建连接管理器（只创建一次连接）
let connection_manager = ClickHouseConnectionManager::new(
    "http://localhost:8123",
    "default", 
    "default", 
    "ClickHouse@123"
)?;

// 使用连接管理器创建数据库实例
let db = connection_manager.create_db();

// 使用连接管理器创建迁移器
let migrator = SimpleMigrator::new(
    "http://localhost:8123",
    "my_service",
    "migrations"
).await?;
```

### 核心组件

1. **`ClickHouseConnectionManager`** - 连接管理器
   - 管理单个 ClickHouse 客户端连接
   - 提供共享连接给多个组件使用
   - 避免重复创建连接

2. **`ClickHouseDB`** - 数据库操作封装
   - 提供基本的数据库操作方法
   - 使用共享连接进行数据库操作

3. **`SimpleMigrator`** - 迁移工具
   - 自动扫描和执行迁移文件
   - 使用共享连接进行迁移操作
   - 支持版本控制和回滚

## 🔧 使用方法

### 基本连接

```rust
use clickhouse_connector::database::ClickHouseConnectionManager;

// 创建连接管理器
let connection_manager = ClickHouseConnectionManager::new(
    "http://localhost:8123",
    "default", 
    "default", 
    "ClickHouse@123"
)?;

// 创建数据库实例
let db = connection_manager.create_db();

// 测试连接
let is_connected = db.test_connection().await?;
```

### 数据库迁移

```rust
use clickhouse_connector::clickhouse_migrator::SimpleMigrator;

// 创建迁移器
let migrator = SimpleMigrator::new(
    "http://localhost:8123",
    "my_service",
    "migrations"
).await?;

// 运行迁移
let result = migrator.migrate().await?;
```

## 📁 迁移文件格式

迁移文件命名格式：`V{版本号}__{描述}.sql`

例如：
- `V000__baseline_existing_database.sql`
- `V001__create_users_table.sql`
- `V002__add_user_status_column.sql`

## 🚀 运行

```bash
# 编译
cargo build

# 运行
cargo run

# 或者使用脚本
./run.sh
```

## 🔍 环境变量

- `CLICKHOUSE_URL` - ClickHouse 服务器地址（默认：http://localhost:8123）
- `CLICKHOUSE_DATABASE` - 数据库名称（默认：default）
- `CLICKHOUSE_USER` - 用户名（默认：default）
- `CLICKHOUSE_PASSWORD` - 密码（默认：ClickHouse@123）

## 💡 优势

1. **避免重复连接** - 使用连接管理器，只创建一次连接
2. **资源共享** - 多个组件共享同一个数据库连接
3. **配置统一** - 所有组件使用相同的连接配置
4. **易于维护** - 连接逻辑集中管理，便于修改和扩展

## 🐛 故障排除

### 连接问题

如果遇到连接问题，请检查：
1. ClickHouse 服务是否运行
2. 网络连接是否正常
3. 认证信息是否正确

### 迁移问题

如果迁移失败，请检查：
1. 迁移文件格式是否正确
2. SQL 语法是否正确
3. 数据库权限是否足够

## 📝 开发说明

### 添加新的数据库操作

在 `ClickHouseDB` 中添加新方法：

```rust
impl ClickHouseDB {
    pub async fn new_operation(&self) -> Result<()> {
        // 使用 self.client 进行操作
        Ok(())
    }
}
```

### 扩展迁移功能

在 `SimpleMigrator` 中添加新功能：

```rust
impl SimpleMigrator {
    pub async fn new_migration_feature(&self) -> Result<()> {
        let client = self.connection_manager.get_client();
        // 使用 client 进行操作
        Ok(())
    }
}
```
