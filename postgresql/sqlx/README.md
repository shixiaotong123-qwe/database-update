# SQLx自动化数据库管理项目

这是一个基于Rust和SQLx的现代化PostgreSQL数据库管理项目，实现了企业级的自动化schema管理和数据迁移功能。

## 🚀 核心特性

### 🔄 自动化迁移系统
- **版本化迁移管理**: 基于SQLx的migration系统
- **现有数据库支持**: 智能检测并建立迁移基线
- **安全迁移执行**: 完整的错误处理和回滚机制
- **状态跟踪**: 详细的迁移历史和执行状态

### 🗄️ 智能数据库管理
- **连接池管理**: 高效的PostgreSQL连接池
- **结构验证**: 自动验证表结构和索引状态
- **数据完整性**: 外键关系和数据一致性检查
- **性能监控**: 库存预警和消费统计

## 表结构

### 1. 用户表 (users)
- id: 主键，自增
- username: 用户名，唯一
- email: 邮箱，唯一
- password_hash: 密码哈希
- full_name: 全名
- created_at: 创建时间
- updated_at: 更新时间
- is_active: 是否激活

### 2. 产品表 (products)
- id: 主键，自增
- name: 产品名称
- description: 产品描述
- price: 价格
- category_id: 分类ID
- stock_quantity: 库存数量
- sku: 产品编码，唯一
- created_at: 创建时间
- updated_at: 更新时间
- is_active: 是否激活

### 3. 订单表 (orders)
- id: 主键，自增
- user_id: 用户ID，外键关联users表
- order_number: 订单号，唯一
- total_amount: 总金额
- status: 订单状态
- shipping_address: 收货地址
- billing_address: 账单地址
- created_at: 创建时间
- updated_at: 更新时间

## 数据库配置

### 环境变量配置：
```bash
DATABASE_URL=postgresql://sxt:default@localhost:5432/postgres
RUST_LOG=info
MIGRATION_FAILURE_STRATEGY=manual
ENABLE_SCHEMA_BACKUP=true
```

### 连接信息：
- **PostgreSQL主配置**: sxt:default@localhost:5432
- **备选OpenGauss**: gaussdb:Welcome1@2024@localhost:5433

## 🚀 快速开始

### 1. 启动数据库
```bash
docker-compose up -d postgresql
```

### 2. 运行项目（推荐）
```bash
./run.sh
```

### 3. 手动运行
```bash
# 设置环境变量（可选，程序有默认值）
export DATABASE_URL="postgresql://sxt:default@localhost:5432/postgres"

# 运行程序
cargo run
```

## 📋 迁移管理

### 添加新迁移
1. 在 `migrations/` 目录创建新的SQL文件：
```bash
# 文件命名格式: 003_description.sql
touch migrations/003_add_new_feature.sql
```

2. 程序会自动检测并执行新迁移

### 迁移文件示例
```sql
-- 003_add_categories_table.sql
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 为现有产品添加分类外键约束
ALTER TABLE products 
ADD CONSTRAINT fk_products_category 
FOREIGN KEY (category_id) REFERENCES categories(id);
```

## 📁 项目结构

```
database_update/
├── migrations/               # 📋 数据库迁移文件
│   ├── 001_initial_schema.sql
│   └── 002_add_user_preferences.sql
├── src/
│   ├── main.rs              # 🚀 主程序入口
│   ├── database.rs          # 🗄️ SQLx数据库管理器
│   ├── tables.rs            # 🔍 表验证和检查模块
│   └── data.rs              # 📊 数据操作模块
├── docker-compose.yml       # 🐳 数据库服务配置
├── run.sh                   # 🛠️ 便捷运行脚本
├── .env                     # ⚙️ 环境变量配置
└── README.md               # 📖 项目文档
```

## 📦 核心依赖

- `sqlx`: 现代化异步SQL工具包，支持编译时查询验证
- `tokio`: 高性能异步运行时
- `anyhow`: 优雅的错误处理
- `tracing`: 结构化日志记录
- `chrono`: 日期时间处理
- `dotenv`: 环境变量管理

## 📊 执行流程

程序启动时自动执行以下步骤：

1. **🔗 数据库连接** - 建立连接池并验证连接
2. **📋 迁移检查** - 检测现有数据库状态  
3. **🔄 自动迁移** - 执行待处理的迁移文件
4. **🔍 结构验证** - 验证表结构和字段类型
5. **📈 数据统计** - 显示数据库状态和统计信息
6. **⚡ 性能检查** - 验证索引状态和外键关系

## 📋 日志示例

```
🚀 开始数据库自动化管理程序
✅ 数据库连接成功
📋 开始执行数据库迁移...
  ✅ v001: initial schema (执行成功)
  ✅ v002: add user preferences (执行成功)
✅ 数据库迁移完成
🔍 验证表结构...
  - users表: 9列 (包含preferences JSONB字段)
  - products表: 10列
  - orders表: 9列
📊 数据统计: 用户5个, 产品10个, 订单10个
📈 数据库自动化管理成功完成!
```

## 🔧 高级功能

### 错误处理
- 迁移失败时自动生成错误报告
- 数据完整性检查和修复建议
- 支持手动/自动回滚策略

### 监控功能
- 库存预警系统
- 用户消费分析
- 订单状态统计
- 索引使用情况监控

## 🎯 最佳实践

1. **迁移文件**：保持迁移文件小而专注，便于回滚
2. **测试优先**：在生产环境前先测试所有迁移
3. **备份策略**：重要迁移前自动创建数据备份
4. **监控告警**：配置Slack/钉钉等告警通知
