-- 创建角色
CREATE ROLE app_readonly;
CREATE ROLE app_readwrite;

-- 授权
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_readonly;
GRANT INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO app_readwrite;

-- 行级安全策略
ALTER TABLE orders ENABLE ROW LEVEL SECURITY;
CREATE POLICY orders_policy ON orders FOR SELECT
USING (user_id = current_setting('app.current_user_id')::INTEGER);