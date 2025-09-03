## sqlx说明

### 1.SQLx 迁移表会自动创建 _sqlx_migrations

CREATE TABLE _sqlx_migrations ( 
    version BIGINT PRIMARY KEY,        -- 从文件名解析的版本号
    description TEXT NOT NULL,         -- 从文件名解析的描述
    installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(), -- 安装时间
    success BOOLEAN NOT NULL,          -- 执行是否成功
    checksum BYTEA NOT NULL,          -- 文件内容校验和
    execution_time BIGINT NOT NULL    -- 执行耗时（纳秒）
);

### 2. 迁移文件名解析规则
<版本号>_<描述>.sql

示例：

migrations/
├── 001_create_users_table.sql        # version=1, description="create users table"
├── 002_add_user_email_index.sql      # version=2, description="add user email index"
├── 20231201_add_products_table.sql   # version=20231201, description="add products table"
└── 003_modify_user_status.sql        # version=3, description="modify user status"

### 3.迁移出错
迁移出错时，此次迁移不生效