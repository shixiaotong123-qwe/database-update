### diesel说明

### 1. Diesel 迁移表会自动创建 __diesel_schema_migrations
CREATE TABLE __diesel_schema_migrations (
    version VARCHAR(50) PRIMARY KEY,    -- 迁移版本号（时间戳格式）
    run_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP  -- 执行时间
);

描述默认不会存储到迁移表中了，若需要存储，需要扩展默认迁移表

### 2. 版本号生成规则

格式：YYYY-MM-DD-HHMMSS_描述
示例：2024-01-15-143022_create_users_table

### 3. CLI 命令回滚

# 回滚最后一个迁移
diesel migration revert

# 回滚到指定的迁移版本
diesel migration revert --version 2024-01-15-143022_create_users_table

# 回滚所有迁移（回到初始状态）
diesel migration revert --all

# 查看将要回滚的迁移（不实际执行）
diesel migration revert --dry-run

# 强制回滚（跳过确认）
diesel migration revert --force

### 4. 注意

# 1. Diesel 要求严格的类型匹配：模型字段必须与数据库表字段完全一致（数量、顺序、类型）

# 2. 使用 diesel print-schema 来确保 schema.rs 与实际数据库结构同步

   diesel print-schema > src/schema.rs

