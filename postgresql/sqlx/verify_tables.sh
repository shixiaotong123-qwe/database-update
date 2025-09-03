#!/bin/bash

echo "🔍 验证PostgreSQL表创建情况"

# 检查PostgreSQL是否运行
if ! docker ps | grep -q postgresql; then
    echo "❌ PostgreSQL数据库未运行"
    echo "请先运行: docker-compose up -d postgresql"
    exit 1
fi

echo "✅ PostgreSQL数据库正在运行"

# 连接数据库并查看表
echo "📋 数据库中的所有表："
docker exec -i postgresql psql -U sxt -d postgres << EOF
\dt
EOF

echo ""
echo "👥 Users表结构："
docker exec -i postgresql psql -U sxt -d postgres << EOF
\d users
EOF

echo ""
echo "📦 Products表结构："
docker exec -i postgresql psql -U sxt -d postgres << EOF
\d products  
EOF

echo ""
echo "📄 Orders表结构："
docker exec -i postgresql psql -U sxt -d postgres << EOF
\d orders
EOF

echo ""
echo "🔍 所有索引："
docker exec -i postgresql psql -U sxt -d postgres << EOF
\di
EOF

echo ""
echo "✨ 验证完成！"
