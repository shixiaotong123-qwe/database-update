-- 创建视图
CREATE VIEW v_active_products AS
SELECT id, product_name, product_price, stock_quantity
FROM products 
WHERE is_active = true;

-- 创建物化视图
CREATE MATERIALIZED VIEW mv_product_stats AS
SELECT category_id, COUNT(*) as product_count, AVG(product_price) as avg_price
FROM products
GROUP BY category_id;

-- 刷新物化视图
REFRESH MATERIALIZED VIEW mv_product_stats;

-- 创建存储过程/函数
CREATE OR REPLACE FUNCTION update_product_stock(product_id INT, quantity_change INT)
RETURNS VOID AS $$
BEGIN
    UPDATE products 
    SET stock_quantity = stock_quantity + quantity_change,
        updated_at = CURRENT_TIMESTAMP
    WHERE id = product_id;
END;
$$ LANGUAGE plpgsql;