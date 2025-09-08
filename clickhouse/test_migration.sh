#!/bin/bash

echo "🧪 ClickHouse 迁移测试脚本"
echo "================================"

echo ""
echo "1️⃣ 基本模式运行（显示基本迁移信息）"
echo "----------------------------------------"
cargo run 2>&1 | head -20

echo ""
echo "2️⃣ 详细模式运行（显示更多错误信息）"
echo "----------------------------------------"
cargo run -- --verbose 2>&1 | head -30

echo ""
echo "3️⃣ 调试模式运行（显示所有日志）"
echo "----------------------------------------"
cargo run -- --debug 2>&1 | head -40

echo ""
echo "✅ 测试完成！"
echo ""
echo "💡 使用说明："
echo "  - 基本模式: cargo run"
echo "  - 详细模式: cargo run -- --verbose"
echo "  - 调试模式: cargo run -- --debug"
echo ""
echo "🔍 如果遇到迁移失败，建议："
echo "  1. 先使用详细模式查看具体错误"
echo "  2. 再使用调试模式查看完整日志"
echo "  3. 检查迁移文件语法和 ClickHouse 连接"
