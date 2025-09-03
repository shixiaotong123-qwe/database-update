-- V002__add_user_status_column.sql

-- +migrate Up
-- 添加状态列
ALTER TABLE users ADD COLUMN IF NOT EXISTS status Enum8('active' = 1, 'inactive' = 2, 'suspended' = 3) DEFAULT 'active';

-- 为现有用户设置默认状态
ALTER TABLE users UPDATE status = 'active' WHERE status = 'active';

-- +migrate Down
ALTER TABLE users DROP COLUMN IF EXISTS status;
