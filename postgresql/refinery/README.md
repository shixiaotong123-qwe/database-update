# Refinery 数据库迁移管理系统

这是一个基于 Refinery 的数据库迁移管理系统，参考了 SQLx 版本的实现，提供了完整的数据库迁移、验证和数据管理功能。

## 特性

- ✅ **自动化迁移管理**: 使用 Refinery 管理数据库版本和迁移
- ✅ **现有数据库支持**: 支持对现有数据库建立迁移基线
- ✅ **完整的验证流程**: 表结构、外键关系、索引状态验证
- ✅ **数据管理**: 自动插入示例数据和统计信息显示
- ✅ **错误处理**: 完善的错误处理和失败报告
- ✅ **日志记录**: 详细的操作日志和进度追踪

## 项目结构

```
postgresql/refinery/
├── Cargo.toml              # 项目配置和依赖
├── .env                    # 环境变量配置
├── run.sh                  # 启动脚本
├── README.md              # 项目文档
├── src/
│   ├── main.rs            # 主程序入口
│   ├── database.rs        # Refinery数据库管理器
│   ├── data.rs           # 数据插入和统计
│   └── tables.rs         # 表结构验证和检查
└── migrations/           # Refinery迁移文件
    ├── V001__initial_schema.sql
    ├── V002__add_user_preferences.sql
    ├── V003__rename_product_name_column.sql
    ├── V004__rename_price_name_column.sql
    ├── V005__delete_sku_column.sql
    ├── V006__add_constraint.sql
    └── V007__add_index.sql
```

## 快速开始

### 1. 环境准备

确保你的系统已安装：
- Rust (rustc 1.70+)
- Docker 和 Docker Compose
- PostgreSQL (通过Docker运行)

### 2. 配置环境变量

项目会自动创建 `.env` 文件，如需修改，请编辑：

```bash
DATABASE_URL=postgresql://sxt:default@localhost:5432/postgres
RUST_LOG=info
```

### 3. 启动数据库

```bash
# 启动PostgreSQL数据库容器
docker-compose up -d postgresql
```

### 4. 运行项目

```bash
# 使用启动脚本（推荐）
chmod +x run.sh
./run.sh

# 或者直接使用cargo
cargo run --bin main
```

## Refinery vs SQLx 对比

| 特性 | Refinery | SQLx |
|------|----------|------|
| 迁移文件格式 | `V{序号}__{描述}.sql` | `{序号}_{描述}.sql` |
| 迁移历史表 | `refinery_schema_history` | `_sqlx_migrations` |
| 配置方式 | Config对象 | 宏和环境变量 |
| 运行时 | tokio-postgres | sqlx-postgres |
| 嵌入迁移 | `embed_migrations!` | `migrate!` |

## 迁移文件说明

### V001__initial_schema.sql
- 创建基础表结构（users, products, orders）
- 建立必要的索引和外键关系

### V002__add_user_preferences.sql  
- 为users表添加JSONB偏好设置字段
- 创建GIN索引支持JSONB查询

### V003-V004__rename_columns.sql
- 重命名产品表的name和price字段
- 演示列重命名操作

### V005__delete_sku_column.sql
- 删除products表的sku列
- 演示列删除操作

### V006__add_constraint.sql
- 添加外键和检查约束
- 增强数据完整性

### V007__add_index.sql
- 创建复合索引和条件索引
- 优化查询性能

## 核心功能

### 1. 数据库连接管理
```rust
// 创建数据库管理器
let db_manager = database::connect().await?;

// 使用自定义URL连接
let db_manager = DatabaseManager::new_with_config(&database_url).await?;
```

### 2. 自动化迁移
```rust
// 执行安全迁移（支持现有数据库）
db_manager.safe_migrate(&database_url).await?;
```

### 3. 表结构验证
```rust
// 验证表结构
tables::validate_tables(db_manager.get_client()).await?;

// 检查外键关系
tables::validate_foreign_keys(db_manager.get_client()).await?;
```

### 4. 数据管理
```rust
// 插入示例数据
data::insert_sample_data(db_manager.get_client()).await?;

// 显示统计信息
data::show_data_statistics(db_manager.get_client()).await?;
```

## 日志输出示例

```
🚀 开始Refinery数据库自动化管理程序
✅ 数据库连接成功
📋 开始执行Refinery数据库迁移...
✅ Refinery数据库迁移完成
已应用的迁移数量: 7
  ✅ 1: initial_schema
  ✅ 2: add_user_preferences
  ✅ 3: rename_product_name_column
...
🔍 验证表结构...
✅ 表结构验证通过
📊 显示数据统计信息...
用户总数: 5
产品总数: 10
订单总数: 10
🎉 程序执行完成
📈 Refinery数据库自动化管理成功完成!
```

## 故障排除

### 1. 数据库连接失败
- 检查PostgreSQL是否运行
- 验证DATABASE_URL配置
- 确认数据库用户权限

### 2. 迁移失败
- 查看详细错误日志
- 检查迁移文件语法
- 验证数据库状态

### 3. 编译错误
- 更新Rust工具链
- 清理编译缓存：`cargo clean`
- 检查依赖版本兼容性

## 开发指南

### 添加新的迁移

1. 在 `migrations/` 目录创建新文件：
   ```bash
   touch migrations/V008__your_migration_name.sql
   ```

2. 编写迁移SQL：
   ```sql
   -- V008__your_migration_name.sql
   -- 描述你的变更
   
   ALTER TABLE users ADD COLUMN phone VARCHAR(20);
   ```

3. 重新编译运行：
   ```bash
   cargo run --bin main
   ```

### 自定义数据处理

可以在 `src/data.rs` 中添加新的数据处理函数：

```rust
pub async fn your_custom_data_function(client: &Client) -> Result<()> {
    // 你的自定义逻辑
    Ok(())
}
```

## 许可证

本项目遵循 MIT 许可证。

