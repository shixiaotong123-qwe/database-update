-- V005__add_user_roles_table.sql

-- +migrate Up
-- TODO: 在这里添加你的迁移 SQL
CREATE TABLE IF NOT EXISTS user_roles (
    user_id UUID,
    role String,
    created_at DateTime64(3) DEFAULT now64(3)
) ENGINE = MergeTree()
ORDER BY user_id;


-- +migrate Down
-- TODO: 在这里添加回滚 SQL
DROP TABLE IF EXISTS user_roles;

