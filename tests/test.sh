#!/bin/bash

# ts-native 自动测试脚本

echo "=== ts-native 自动测试 ==="
echo ""

PASS=0
FAIL=0

run_test() {
    local file=$1
    local expected=$2
    
    echo -n "测试 $file... "
    
    # 编译
    cargo run --quiet -- $file > /dev/null 2>&1
    if [ $? -ne 0 ]; then
        echo "❌ 编译失败"
        FAIL=$((FAIL + 1))
        return
    fi
    
    # 运行并获取输出
    output=$(./a.exe 2>&1)
    
    # 检查输出
    if [ "$output" == "$expected" ]; then
        echo "✅ 通过"
        PASS=$((PASS + 1))
    else
        echo "❌ 失败"
        echo "   预期: $expected"
        echo "   实际: $output"
        FAIL=$((FAIL + 1))
    fi
}

# 基础测试
run_test test_simple_num.ts "42"
run_test test_arithmetic.ts "30
35
42
25"
run_test test_comparison.ts "1
1
10"

# 字符串测试
run_test test_string.ts "Hello"
run_test test_string_concat.ts "Hello World"

# 数组测试
run_test test_array.ts "10
20
30"
run_test test_array_multi.ts "10
20
30"

# 对象测试
run_test test_object.ts "10"
run_test test_object_multi.ts "10
20
30"

# 函数测试
run_test test_factorial.ts "120"
run_test test_fib.ts "55"

# 控制流测试
run_test test_while.ts "0
1
2
3
4"
run_test test_for.ts "0
1
2
3
4"

# 三元运算符测试
run_test test_ternary.ts "100"

# typeof 测试
run_test test_typeof.ts "number
string
object
object
boolean
object"

echo ""
echo "=== 测试结果 ==="
echo "通过: $PASS"
echo "失败: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "🎉 所有测试通过！"
    exit 0
else
    echo "⚠️  有测试失败"
    exit 1
fi
