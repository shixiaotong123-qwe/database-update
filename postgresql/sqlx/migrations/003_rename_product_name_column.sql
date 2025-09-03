-- 003_rename_product_name_column.sql
-- 将products表的name列重命名为product_name

-- 重命名列：name -> product_name
ALTER TABLE products RENAME COLUMN name TO product_name;
