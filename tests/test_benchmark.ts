// 性能基准测试

// 斐波那契数列（递归）
function fib_recursive(n) {
    if (n <= 1) {
        return n;
    }
    return fib_recursive(n - 1) + fib_recursive(n - 2);
}

// 斐波那契数列（迭代）
function fib_iterative(n) {
    if (n <= 1) {
        return n;
    }
    let a = 0;
    let b = 1;
    let i = 2;
    while (i <= n) {
        let temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }
    return b;
}

// 阶乘（递归）
function factorial_recursive(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial_recursive(n - 1);
}

// 阶乘（迭代）
function factorial_iterative(n) {
    let result = 1;
    let i = 2;
    while (i <= n) {
        result = result * i;
        i = i + 1;
    }
    return result;
}

// 数组求和
function array_sum(arr, n) {
    let sum = 0;
    for (let i = 0; i < n; i = i + 1) {
        sum = sum + arr[i];
    }
    return sum;
}

// 数组最大值
function array_max(arr, n) {
    let max_val = arr[0];
    for (let i = 1; i < n; i = i + 1) {
        if (arr[i] > max_val) {
            max_val = arr[i];
        }
    }
    return max_val;
}

function main() {
    print("=== Performance Benchmark ===");
    
    // 斐波那契测试
    print(fib_recursive(20));
    print(fib_iterative(20));
    
    // 阶乘测试
    print(factorial_recursive(12));
    print(factorial_iterative(12));
    
    // 数组操作测试
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    print(array_sum(arr, 10));
    print(array_max(arr, 10));
    
    // 字符串操作测试
    let s1 = "Hello";
    let s2 = "World";
    let s3 = s1 + " " + s2;
    print(s3);
    
    // 对象操作测试
    let obj = { x: 100, y: 200, z: 300 };
    print(obj.x + obj.y + obj.z);
    
    print("=== Benchmark Complete ===");
    
    return 0;
}
