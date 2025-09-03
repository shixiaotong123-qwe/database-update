-- 添加外键约束
ALTER TABLE orders ADD CONSTRAINT fk_orders_user 
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

-- 添加检查约束
ALTER TABLE products ADD CONSTRAINT chk_price_positive 
CHECK (product_price > 0);
