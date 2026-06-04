#!/bin/bash
# ts-native 编译和运行脚本

set -e

echo "=== ts-native 构建脚本 ==="
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查参数
if [ $# -lt 1 ]; then
    echo "用法: $0 <input.ts>"
    echo "      $0 --test"
    exit 1
fi

# 构建编译器
echo -e "${YELLOW}1. 编译 ts-native...${NC}"
cargo build --release
echo -e "${GREEN}✅ 编译器构建完成${NC}"
echo ""

# 运行测试或编译文件
if [ "$1" == "--test" ]; then
    echo -e "${YELLOW}2. 运行内置测试...${NC}"
    cargo run --release -- --test
else
    INPUT_FILE=$1
    OUTPUT_FILE="${INPUT_FILE%.ts}.o"
    EXE_FILE="${INPUT_FILE%.ts}.exe"
    
    echo -e "${YELLOW}2. 编译 $INPUT_FILE...${NC}"
    cargo run --release -- "$INPUT_FILE" -o "$OUTPUT_FILE"
    echo ""
    
    # 检查目标文件
    if [ -f "$OUTPUT_FILE" ]; then
        SIZE=$(stat -f%z "$OUTPUT_FILE" 2>/dev/null || stat -c%s "$OUTPUT_FILE" 2>/dev/null)
        echo -e "${GREEN}✅ 生成目标文件: $OUTPUT_FILE ($SIZE bytes)${NC}"
        
        # 显示符号表
        if command -v nm &> /dev/null; then
            echo ""
            echo -e "${YELLOW}符号表:${NC}"
            nm "$OUTPUT_FILE" 2>/dev/null || echo "  (无法读取符号表)"
        fi
    fi
    
    # 尝试链接
    if [ -f "$EXE_FILE" ]; then
        echo ""
        echo -e "${GREEN}✅ 生成可执行文件: $EXE_FILE${NC}"
        
        # 运行
        echo ""
        echo -e "${YELLOW}3. 运行程序...${NC}"
        ./"$EXE_FILE"
    fi
fi

echo ""
echo -e "${GREEN}=== 完成 ===${NC}"
