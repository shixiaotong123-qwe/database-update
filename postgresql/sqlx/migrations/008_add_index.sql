-- 创建复合索引
CREATE INDEX idx_orders_user_status ON orders(user_id, status);

-- 创建部分索引（条件索引）
CREATE INDEX idx_active_products ON products(product_name) WHERE is_active = true;
