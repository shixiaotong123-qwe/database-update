-- 扩大列长度
ALTER TABLE users ALTER COLUMN username TYPE VARCHAR(100);

-- 变更数据类型（需要类型转换）
ALTER TABLE products ALTER COLUMN product_price TYPE NUMERIC(12,4) USING product_price::NUMERIC(12,4);

-- 修改列的默认值
ALTER TABLE users ALTER COLUMN is_active SET DEFAULT false;

-- 修改列的NULL约束
ALTER TABLE users ALTER COLUMN full_name SET NOT NULL;