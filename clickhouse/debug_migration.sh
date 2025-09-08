#!/bin/bash

echo "🔍 ClickHouse 迁移调试工具"
echo "================================"

# 检查迁移状态
echo "📊 检查迁移状态..."
cargo run 2>&1 | grep -E "(迁移状态|成功迁移数|失败迁移数|失败的迁移详情|成功的迁移)"

echo ""
echo "🔍 检查失败的迁移..."
cargo run 2>&1 | grep -A 10 "失败的迁移详情" || echo "没有找到失败的迁移详情"

echo ""
echo "📋 检查迁移表结构..."
echo "连接到 ClickHouse 并检查迁移表..."

# 这里可以添加更多的调试命令
echo "建议的调试步骤："
echo "1. 检查迁移文件语法是否正确"
echo "2. 验证 ClickHouse 连接是否正常"
echo "3. 查看具体的错误日志"
echo "4. 检查迁移表的记录"
echo "5. 验证 SQL 语句是否兼容 ClickHouse"
