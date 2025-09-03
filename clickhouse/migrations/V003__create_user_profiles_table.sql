-- V003__create_user_profiles_table.sql

-- +migrate Up
-- 创建新的用户资料表
CREATE TABLE IF NOT EXISTS user_profiles (
    user_id UUID,
    first_name String,
    last_name String,
    avatar_url String DEFAULT '',
    bio String DEFAULT '',
    created_at DateTime64(3) DEFAULT now64(3)
) ENGINE = MergeTree()
ORDER BY user_id;

-- 注意：这里不插入数据，因为users表可能还是空的
-- 数据插入应该在后续的迁移中处理，或者通过应用程序逻辑处理

-- +migrate Down
DROP TABLE IF EXISTS user_profiles;
