#!/bin/bash

echo "🚀 启动Refinery数据库自动化管理项目"

# 检查是否安装了Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到Rust/Cargo，请先安装Rust"
    echo "   安装地址: https://rustup.rs/"
    exit 1
fi

# 检查环境变量配置
if [ ! -f ".env" ]; then
    echo "⚠️  未找到.env文件，使用默认配置..."
    echo "DATABASE_URL=postgresql://sxt:default@localhost:5432/postgres" > .env
    echo "RUST_LOG=info" >> .env
    echo "✅ 已创建默认.env文件"
fi

# 检查数据库是否运行
echo "🔍 检查PostgreSQL数据库状态..."
if ! docker ps | grep -q postgresql; then
    echo "⚠️  PostgreSQL数据库未运行，正在启动..."
    docker-compose up -d postgresql
    echo "⏳ 等待数据库启动..."
    sleep 5
fi

# 检查迁移文件
echo "📋 检查Refinery迁移文件..."
if [ -d "migrations" ]; then
    migration_count=$(ls -1 migrations/V*.sql 2>/dev/null | wc -l)
    echo "✅ 发现 $migration_count 个Refinery迁移文件"
else
    echo "❌ 错误: 未找到migrations目录"
    exit 1
fi

# 编译并运行项目
echo "🔨 编译Refinery项目..."
cargo build

if [ $? -eq 0 ]; then
    echo "✅ 编译成功，开始执行Refinery自动化数据库管理..."
    echo "📊 程序将自动执行以下操作:"
    echo "   - 🔗 使用Refinery连接数据库"
    echo "   - 📋 检查和执行Refinery迁移"
    echo "   - 🔍 验证表结构"
    echo "   - 📈 显示数据统计"
    echo ""
    cargo run --bin main
else
    echo "❌ 编译失败"
    exit 1
fi

