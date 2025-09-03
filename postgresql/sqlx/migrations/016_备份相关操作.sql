-- 创建备份表
CREATE TABLE users_backup AS SELECT * FROM users;

-- 基于时间点的备份
CREATE TABLE orders_before_migration AS 
SELECT * FROM orders WHERE created_at < '2024-01-01';