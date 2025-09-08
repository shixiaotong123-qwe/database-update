# ClickHouse 数据库连接器和迁移工具

这是一个用于 ClickHouse 数据库的 Rust 连接器和迁移工具，支持自动化数据库迁移管理。

## 功能特性

- 🔌 ClickHouse 数据库连接管理
- 🚀 自动化数据库迁移
- 📊 迁移状态跟踪
- 🔍 详细的错误诊断和日志记录
- 📝 迁移文件版本管理
- ✅ 迁移校验和验证

## 快速开始

### 1. 启动 ClickHouse 服务

```bash
# 使用 Docker 启动 ClickHouse
docker run -d --name clickhouse-server \
  -p 8123:8123 -p 9000:9000 \
  clickhouse/clickhouse-server:latest
```

### 2. 运行迁移工具

```bash
# 基本运行
cargo run

# 启用详细模式（显示更多错误信息）
cargo run -- --verbose

# 启用调试模式（显示所有日志）
cargo run -- --debug
```

## 迁移文件格式

迁移文件应遵循以下命名约定：
```
V001__create_users_table.sql
V002__add_user_status_column.sql
V003__create_user_profiles_table.sql
```

## 错误排查指南

### 常见问题

#### 1. 迁移失败但没有明确错误信息

**解决方案：**
- 使用详细模式运行：`cargo run -- --verbose`
- 使用调试模式运行：`cargo run -- --debug`
- 检查迁移文件语法是否正确
- 验证 ClickHouse 连接是否正常

#### 2. SQL 执行错误

**排查步骤：**
1. 检查 SQL 语句是否兼容 ClickHouse
2. 验证表结构是否正确
3. 查看详细的错误日志
4. 检查权限设置

#### 3. 连接问题

**排查步骤：**
1. 确认 ClickHouse 服务正在运行
2. 检查端口配置（默认：8123）
3. 验证用户名和密码
4. 检查网络连接

### 调试工具

#### 使用详细模式
```bash
cargo run -- --verbose
```
详细模式会显示：
- 每个迁移的执行状态
- 失败的迁移详情
- 数据库中的失败记录
- 执行时间和错误信息

#### 使用调试模式
```bash
cargo run -- --debug
```
调试模式会显示：
- 所有日志信息
- SQL 语句执行详情
- 迁移文件解析过程
- 数据库操作详情

#### 调试脚本
```bash
# 运行调试脚本
./debug_migration.sh
```

## 项目结构

```
clickhouse/
├── src/
│   ├── main.rs                 # 主程序入口
│   ├── lib.rs                  # 库入口
│   ├── database.rs             # 数据库连接管理
│   ├── models.rs               # 数据模型
│   └── clickhouse_migrator/    # 迁移器实现
│       ├── mod.rs              # 模块定义
│       └── simple_migrator.rs  # 简单迁移器
├── migrations/                  # 迁移文件目录
├── Cargo.toml                  # 项目配置
└── README.md                   # 项目说明
```

## 配置说明

### 环境变量

- `CONTINUE_ON_MIGRATION_FAILURE`: 设置为 "true" 时，迁移失败后继续执行其他迁移

### 数据库连接

默认连接配置：
- 主机：localhost:8123
- 数据库：default
- 用户名：default
- 密码：ClickHouse@123

## 开发指南

### 添加新的迁移

1. 在 `migrations/` 目录下创建新的 SQL 文件
2. 使用正确的命名格式：`V{版本号}__{描述}.sql`
3. 编写 SQL 语句
4. 运行迁移工具

### 自定义迁移器

可以继承 `SimpleMigrator` 类来创建自定义迁移器：

```rust
use clickhouse_connector::clickhouse_migrator::SimpleMigrator;

pub struct CustomMigrator {
    base: SimpleMigrator,
    // 自定义字段
}

impl CustomMigrator {
    pub async fn new(database_url: &str, service_name: &str, migrations_path: &str) -> Result<Self> {
        let base = SimpleMigrator::new(database_url, service_name, migrations_path).await?;
        Ok(Self { base })
    }
}
```

## 故障排除

### 日志级别

- `ERROR`: 错误信息
- `WARN`: 警告信息  
- `INFO`: 一般信息
- `DEBUG`: 调试信息

### 常见错误码

- 连接错误：检查 ClickHouse 服务状态
- SQL 语法错误：验证 SQL 语句
- 权限错误：检查用户权限
- 表不存在：确认表结构

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 推送到分支
5. 创建 Pull Request

## 许可证

MIT License
