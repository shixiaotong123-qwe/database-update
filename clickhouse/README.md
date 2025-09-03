# ClickHouse 数据库连接器

一个现代化的 ClickHouse 数据库连接器，支持自动数据库迁移管理。

## ✨ 特性

- 🔄 **自动迁移管理** - 程序启动时自动检查并执行待处理的迁移文件
- 🚀 **零配置启动** - 使用合理的默认配置，开箱即用
- 🔧 **灵活配置** - 支持环境变量配置
- 📊 **实时状态监控** - 显示迁移执行状态和结果
- 🛡️ **错误处理** - 优雅处理迁移失败情况

## 🚀 快速开始

### 1. 安装依赖

```bash
# 确保已安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆项目
git clone <your-repo>
cd clickhouse
```

### 2. 配置环境变量（可选）

创建 `.env` 文件：

```bash
# ClickHouse 数据库连接配置
CLICKHOUSE_URL=http://localhost:8123
CLICKHOUSE_SERVICE=clickhouse_service
CLICKHOUSE_MIGRATIONS=./migrations
```

### 3. 运行程序

```bash
# 方式1: 使用脚本（推荐）
./run.sh

# 方式2: 直接运行
cargo run

# 方式3: 指定二进制目标
cargo run --bin clickhouse_connector
```

## 🔄 自动迁移机制

### 工作原理

1. **程序启动时**自动检查 `./migrations/` 目录
2. **扫描迁移文件**，识别待执行的迁移
3. **按版本顺序执行**，确保迁移顺序正确
4. **记录执行状态**，避免重复执行
5. **显示执行结果**，包括成功和失败的迁移

### 迁移文件格式

迁移文件命名格式：`V{版本号}__{描述}.sql`

```sql
-- V001__create_users_table.sql

-- +migrate Up
CREATE TABLE users (
    id UInt32,
    name String,
    email String
) ENGINE = MergeTree()
ORDER BY id;

-- +migrate Down
DROP TABLE users;
```

### 迁移状态管理

程序会自动创建 `migrations` 表来跟踪迁移状态：

```sql
CREATE TABLE migrations (
    version String,
    name String,
    executed_at DateTime,
    checksum String
) ENGINE = MergeTree()
ORDER BY version;
```

## 📁 项目结构

```
clickhouse/
├── src/
│   ├── main.rs                 # 主程序入口（包含自动迁移逻辑）
│   ├── lib.rs                  # 库入口
│   ├── clickhouse_migrator/    # 迁移器实现
│   ├── database.rs             # 数据库连接逻辑
│   └── models.rs               # 数据模型
├── migrations/                 # 迁移文件目录
├── run.sh                     # 运行脚本
└── Cargo.toml                 # 项目配置
```

## 🔧 配置选项

### 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `CLICKHOUSE_URL` | `http://localhost:8123` | ClickHouse 服务器地址 |
| `CLICKHOUSE_SERVICE` | `clickhouse_service` | 服务标识符 |
| `CLICKHOUSE_MIGRATIONS` | `./migrations` | 迁移文件目录 |

### 命令行参数

程序现在**不需要**任何命令行参数，所有操作都是自动的。

## 📊 输出示例

```
🚀 ClickHouse 数据库连接器启动中...
🔧 配置信息:
  数据库 URL: http://localhost:8123
  服务名称: clickhouse_service
  迁移路径: ./migrations

正在连接ClickHouse数据库...
✅ 成功连接到ClickHouse数据库！

📊 数据库信息:
数据库名称: default
用户名: default
主机: http://localhost:8123

🚀 自动检查并执行数据库迁移...
📊 当前迁移状态:
  服务名称: clickhouse_service
  迁移表: migrations
  总迁移数: 5

✅ 没有待处理的迁移，数据库已是最新状态

🔍 查询表结构...
...
🎉 ClickHouse数据库操作完成！

💡 提示: 程序启动时会自动检查并执行迁移，无需手动操作
```

## 🚨 故障排除

### 常见问题

1. **连接失败**
   - 检查 ClickHouse 服务是否运行
   - 验证连接 URL 和认证信息

2. **迁移失败**
   - 检查迁移文件语法
   - 查看错误日志
   - 确保数据库权限足够

3. **权限问题**
   - 确保用户有创建表的权限
   - 检查数据库访问权限

## 🔮 未来计划

- [ ] 支持迁移回滚
- [ ] 添加迁移验证
- [ ] 支持分布式迁移
- [ ] 添加迁移测试框架

## �� 许可证

MIT License
