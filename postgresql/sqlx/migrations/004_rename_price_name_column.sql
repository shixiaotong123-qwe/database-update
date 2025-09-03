-- 004_rename_price_name_column.sql
-- 将products表的price列重命名为product_price

-- 重命名列：price -> product_price
ALTER TABLE products RENAME COLUMN price TO product_price;
