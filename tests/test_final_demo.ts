// ts-native 编译器 - 完整功能演示

// ==================== 数学函数 ====================
function abs(x) {
    if (x < 0) {
        return 0 - x;
    }
    return x;
}

function max(a, b) {
    if (a > b) {
        return a;
    }
    return b;
}

function min(a, b) {
    if (a < b) {
        return a;
    }
    return b;
}

function power(base, exp) {
    if (exp == 0) {
        return 1;
    }
    return base * power(base, exp - 1);
}

// ==================== 数组操作 ====================
function array_sum(arr, n) {
    let sum = 0;
    for (let i = 0; i < n; i = i + 1) {
        sum = sum + arr[i];
    }
    return sum;
}

function array_max(arr, n) {
    let m = arr[0];
    for (let i = 1; i < n; i = i + 1) {
        if (arr[i] > m) {
            m = arr[i];
        }
    }
    return m;
}

// ==================== 字符串操作 ====================
function greet(name) {
    return "Hello, " + name + "!";
}

// ==================== 对象操作 ====================
function create_rect(w, h) {
    return { width: w, height: h };
}

function rect_area(r) {
    return r.width * r.height;
}

function rect_perimeter(r) {
    return 2 * (r.width + r.height);
}

// ==================== 主函数 ====================
function main() {
    print("=== ts-native Demo ===");
    
    // 数学运算
    print(power(2, 10));
    print(abs(-42));
    print(max(3, 7));
    print(min(3, 7));
    
    // 数组操作
    let nums = [10, 20, 30, 40, 50];
    print(array_sum(nums, 5));
    print(array_max(nums, 5));
    
    // 字符串操作
    let msg = greet("World");
    print(msg);
    
    // 对象操作
    let rect = create_rect(5, 3);
    print(rect_area(rect));
    print(rect_perimeter(rect));
    
    // 类型检测
    print(typeof 42);
    print(typeof "hello");
    print(typeof nums);
    print(typeof rect);
    
    // 三元运算符
    let value = -10;
    let absolute = value < 0 ? 0 - value : value;
    print(absolute);
    
    // 逻辑运算
    let result = (1 && 1) || 0;
    print(result);
    
    print("=== Test Complete ===");
    
    return 0;
}
