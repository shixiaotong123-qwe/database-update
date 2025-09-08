-- V002__add_user_preferences.sql
-- 示例迁移：为用户表添加偏好设置字段

ALTER TABLE users 
ADD COLUMN IF NOT EXISTS preferences JSONB DEFAULT '{}';

-- 为偏好设置创建索引以支持JSONB查询
CREATE INDEX IF NOT EXISTS idx_users_preferences_gin ON users USING GIN (preferences);

-- 更新现有用户的偏好设置为默认值（如果需要）
UPDATE users 
SET preferences = '{
    "theme": "light",
    "language": "zh-CN",
    "notifications": true
}'::jsonb
WHERE preferences IS NULL OR preferences = '{}'::jsonb;

