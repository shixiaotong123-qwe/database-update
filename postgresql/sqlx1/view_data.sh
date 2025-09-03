#!/bin/bash

echo "📊 查看数据库中的数据"

# 检查PostgreSQL是否运行
if ! docker ps | grep -q postgresql; then
    echo "❌ PostgreSQL数据库未运行"
    echo "请先运行: docker-compose up -d postgresql"
    exit 1
fi

echo "✅ PostgreSQL数据库正在运行"

echo ""
echo "👥 用户数据："
docker exec -i postgresql psql -U sxt -d postgres << EOF
SELECT id, username, email, full_name, is_active, created_at FROM users ORDER BY id;
EOF

echo ""
echo "📦 产品数据："
docker exec -i postgresql psql -U sxt -d postgres << EOF
SELECT id, name, price, category_id, stock_quantity, sku, is_active FROM products ORDER BY id LIMIT 10;
EOF

echo ""
echo "📄 订单数据："
docker exec -i postgresql psql -U sxt -d postgres << EOF
SELECT id, user_id, order_number, total_amount, status, created_at FROM orders ORDER BY id;
EOF

echo ""
echo "📈 统计信息："
docker exec -i postgresql psql -U sxt -d postgres << EOF
SELECT 'Users' as table_name, COUNT(*) as count FROM users
UNION ALL
SELECT 'Products' as table_name, COUNT(*) as count FROM products
UNION ALL
SELECT 'Orders' as table_name, COUNT(*) as count FROM orders;
EOF

echo ""
echo "📊 订单状态统计："
docker exec -i postgresql psql -U sxt -d postgres << EOF
SELECT status, COUNT(*) as count FROM orders GROUP BY status ORDER BY count DESC;
EOF

echo ""
echo "💰 订单金额统计："
docker exec -i postgresql psql -U sxt -d postgres << EOF
SELECT 
    status,
    COUNT(*) as order_count,
    ROUND(SUM(total_amount), 2) as total_amount,
    ROUND(AVG(total_amount), 2) as avg_amount
FROM orders 
GROUP BY status 
ORDER BY total_amount DESC;
EOF

echo ""
echo "🔗 用户订单关联查询："
docker exec -i postgresql psql -U sxt -d postgres << EOF
SELECT 
    u.full_name,
    COUNT(o.id) as order_count,
    ROUND(SUM(o.total_amount), 2) as total_spent
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
GROUP BY u.id, u.full_name
ORDER BY total_spent DESC;
EOF

echo ""
echo "✨ 查看完成！"
