#!/bin/bash
# 编译脚本 - Linux/macOS

echo "编译 TS-Native 运维采集CLI..."

# 编译为原生二进制
ts-native main.ts

# 重命名输出
if [ -f "a.out" ]; then
    mv a.out metric-collector
    chmod +x metric-collector
    echo "✅ 编译成功: metric-collector"
    ls -lh metric-collector
else
    echo "❌ 编译失败"
    exit 1
fi