-- 数据清理
DELETE FROM orders WHERE status = 'cancelled' AND created_at < NOW() - INTERVAL '1 year';

-- 数据迁移（分割列）
ALTER TABLE users ADD COLUMN first_name VARCHAR(50);
ALTER TABLE users ADD COLUMN last_name VARCHAR(50);

UPDATE users 
SET first_name = SPLIT_PART(full_name, ' ', 1),
    last_name = SPLIT_PART(full_name, ' ', 2)
WHERE full_name IS NOT NULL;

-- 数据标准化
UPDATE users SET email = LOWER(TRIM(email)) WHERE email IS NOT NULL;

-- 批量数据更新
UPDATE products 
SET category_id = 1 
WHERE category_id IS NULL AND product_name ILIKE '%电子%';