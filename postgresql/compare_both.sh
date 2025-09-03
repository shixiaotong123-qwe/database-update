#!/bin/bash

echo "🚀 数据库 ORM 对比测试：SQLx vs Diesel"
echo "=============================================="

# 检查是否安装了Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到Rust/Cargo，请先安装Rust"
    echo "   安装地址: https://rustup.rs/"
    exit 1
fi

# 检查数据库是否运行
echo "🔍 检查PostgreSQL数据库状态..."
if ! docker ps | grep -q postgresql; then
    echo "⚠️  PostgreSQL数据库未运行，正在启动..."
    docker-compose up -d postgresql
    echo "⏳ 等待数据库启动..."
    sleep 8
fi

echo ""
echo "📋 开始 SQLx 版本测试"
echo "====================="
cd sqlx
echo "🔨 编译 SQLx 项目..."
cargo build --quiet

if [ $? -eq 0 ]; then
    echo "✅ SQLx 编译成功"
    echo "▶️  运行 SQLx 数据库管理程序..."
    cargo run --quiet
    echo ""
else
    echo "❌ SQLx 编译失败"
    exit 1
fi

cd ..

echo ""
echo "📋 开始 Diesel 版本测试"  
echo "======================="
cd diesel
echo "🔨 编译 Diesel 项目..."
cargo build --quiet

if [ $? -eq 0 ]; then
    echo "✅ Diesel 编译成功"
    echo "▶️  运行 Diesel 数据库管理程序..."
    DATABASE_URL=postgresql://sxt:default@localhost:5432/postgres1 cargo run --quiet
    echo ""
else
    echo "❌ Diesel 编译失败"
    exit 1
fi

cd ..

echo ""
echo "📊 测试结果对比"
echo "=============="
echo "SQLx 数据库 (postgres):"
psql postgresql://sxt:default@localhost:5432/postgres -c "SELECT COUNT(*) as user_count FROM users;" -t
echo "用户数"

echo "Diesel 数据库 (postgres1):"  
psql postgresql://sxt:default@localhost:5432/postgres1 -c "SELECT COUNT(*) as user_count FROM users;" -t
echo "用户数"

echo ""
echo "🎉 对比测试完成！"
echo ""
echo "📖 查看详细分析报告："
echo "   cat SQLx_vs_Diesel_比较分析.md"
echo ""
echo "📁 项目结构："
echo "   sqlx/    - SQLx 版本实现"
echo "   diesel/  - Diesel 版本实现"
