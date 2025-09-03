#!/bin/bash

echo "🔍 调试 ClickHouse 迁移检测"
echo "================================"

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
echo "📁 检查迁移文件..."
echo "迁移目录: ./migrations"
ls -la ./migrations/

echo ""
echo "🚀 运行程序（将显示详细的迁移检测信息）..."
echo "💡 观察输出，应该能看到："
echo "  - 扫描到的迁移文件列表"
echo "  - 迁移表状态检查"
echo "  - 待处理的迁移列表"
echo ""

# 运行程序
cargo run

echo ""
echo "🎯 调试完成！"
echo "💡 如果看到 'V006__改变数据类型' 被检测到，说明问题已解决"
