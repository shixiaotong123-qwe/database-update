-- 删除索引
DROP INDEX IF EXISTS idx_active_products;

-- 重建索引（性能优化）
REINDEX INDEX idx_orders_user_id;