#!/bin/bash

echo "🚀 ClickHouse 数据库连接器"
echo "================================"

# 检查是否安装了 Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ 错误: 未找到 Rust 和 Cargo"
    echo "请先安装 Rust: https://rustup.rs/"
    exit 1
fi

echo "✅ Rust 环境检查通过"

# 构建项目
echo "🔨 正在构建项目..."
if cargo build; then
    echo "✅ 项目构建成功"
else
    echo "❌ 项目构建失败"
    exit 1
fi

# 运行主程序（现在会自动执行迁移）
echo "🚀 运行主程序..."
echo "💡 程序启动时会自动检查并执行数据库迁移"
cargo run

echo ""
echo "🎉 程序执行完成！"
echo ""
echo "💡 提示:"
echo "  - 程序启动时会自动执行迁移，无需手动操作"
echo "  - 如需查看迁移状态，可以检查程序输出"
echo "  - 所有迁移文件位于 ./migrations/ 目录"
