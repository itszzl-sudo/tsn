#!/bin/bash

# ts-native 发布到 crates.io 脚本

echo "=== ts-native 发布脚本 ==="
echo ""
echo "步骤 1: 检查 cargo 配置"
echo "当前使用了镜像源，需要临时切换到官方源"
echo ""

# 备份原配置
if [ -f ~/.cargo/config.toml ]; then
    cp ~/.cargo/config.toml ~/.cargo/config.toml.backup
    echo "✅ 已备份配置到 ~/.cargo/config.toml.backup"
fi

# 创建临时配置（使用官方源）
cat > ~/.cargo/config.toml.publish << 'EOF'
# 临时配置 - 用于发布到 crates.io
[net]
git-fetch-with-cli = true

[http]
check-revoke = false
timeout = 60

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
EOF

echo ""
echo "步骤 2: 登录 crates.io"
echo "如果没有 token，请访问: https://crates.io/settings/tokens"
echo ""
read -p "请输入 crates.io API token: " token

if [ -z "$token" ]; then
    echo "❌ 未提供 token，退出"
    exit 1
fi

echo "$token" | cargo login --registry crates-io

echo ""
echo "步骤 3: 发布到 crates.io"
echo ""

# 使用临时配置发布
mv ~/.cargo/config.toml ~/.cargo/config.toml.mirror
mv ~/.cargo/config.toml.publish ~/.cargo/config.toml

cargo publish

# 恢复原配置
mv ~/.cargo/config.toml ~/.cargo/config.toml.publish
mv ~/.cargo/config.toml.mirror ~/.cargo/config.toml

echo ""
echo "✅ 发布完成！"
echo ""
echo "包地址: https://crates.io/crates/ts-native"
echo ""
echo "恢复原镜像配置..."
