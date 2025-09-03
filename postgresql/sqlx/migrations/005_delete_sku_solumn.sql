-- 005_delete_sku_solumn.sql
-- 删除products表的sku列

-- 删除列：sku
ALTER TABLE products DROP COLUMN sku;
