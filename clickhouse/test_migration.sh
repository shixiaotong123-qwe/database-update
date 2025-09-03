#!/bin/bash

echo "🧪 测试 ClickHouse 自动迁移功能"
echo "=================================="

# 检查 ClickHouse 服务是否运行
echo "🔍 检查 ClickHouse 服务状态..."
if curl -s "http://localhost:8123" > /dev/null; then
    echo "✅ ClickHouse 服务正在运行"
else
    echo "❌ ClickHouse 服务未运行，请先启动服务"
    echo "💡 提示: 可以使用 Docker 快速启动"
    echo "   docker run -d --name clickhouse-server -p 8123:8123 -p 9000:9000 clickhouse/clickhouse-server:latest"
    exit 1
fi

echo ""
echo "🚀 运行程序（将自动执行迁移）..."
echo "💡 观察输出，应该能看到自动迁移过程"
echo ""

# 运行程序
cargo run

echo ""
echo "🎯 测试完成！"
echo "💡 如果看到 '✅ 没有待处理的迁移' 或 '✅ 所有迁移执行成功'，说明自动迁移功能正常"
