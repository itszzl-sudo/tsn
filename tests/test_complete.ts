// ts-native 完整特性测试

// 递归函数
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

// 多参数函数
function max3(a, b, c) {
    let m = a;
    if (b > m) {
        m = b;
    }
    if (c > m) {
        m = c;
    }
    return m;
}

// 数组处理
function array_sum(arr, n) {
    let sum = 0;
    for (let i = 0; i < n; i = i + 1) {
        sum = sum + arr[i];
    }
    return sum;
}

// 对象操作
function describe_point(p) {
    return p.x + p.y;
}

function main() {
    // 字符串测试
    let greeting = "Hello" + " " + "World";
    print(greeting);
    
    // 数字运算
    print(factorial(5));
    print(max3(3, 7, 5));
    
    // 数组测试
    let nums = [10, 20, 30, 40, 50];
    print(array_sum(nums, 5));
    
    // 对象测试
    let point = { x: 100, y: 200 };
    print(describe_point(point));
    
    // 三元运算符
    let abs_val = -42;
    let abs = abs_val < 0 ? 0 - abs_val : abs_val;
    print(abs);
    
    // 逻辑运算
    let result = (1 && 1) || 0;
    print(result);
    
    // 嵌套数组
    let matrix = [[1, 2], [3, 4]];
    print(matrix[0][0] + matrix[1][1]);
    
    return 0;
}
