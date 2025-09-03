-- V001__create_users_table.sql

-- +migrate Up
CREATE TABLE IF NOT EXISTS users (
    id UUID DEFAULT generateUUIDv4(),
    username String,
    email String,
    password_hash String,
    created_at DateTime64(3) DEFAULT now64(3),
    updated_at DateTime64(3) DEFAULT now64(3)
) ENGINE = MergeTree()
ORDER BY id
SETTINGS index_granularity = 8192;

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_users_email ON users (email) TYPE bloom_filter GRANULARITY 1;
CREATE INDEX IF NOT EXISTS idx_users_username ON users (username) TYPE bloom_filter GRANULARITY 1;

-- +migrate Down
DROP INDEX IF EXISTS idx_users_username ON users;
DROP INDEX IF EXISTS idx_users_email ON users;
DROP TABLE IF EXISTS users;
