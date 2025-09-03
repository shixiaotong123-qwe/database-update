-- 删除约束
ALTER TABLE products DROP CONSTRAINT chk_price_positive;

-- 修改约束（先删除再添加）
ALTER TABLE orders DROP CONSTRAINT fk_orders_user;
ALTER TABLE orders ADD CONSTRAINT fk_orders_user 
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL;