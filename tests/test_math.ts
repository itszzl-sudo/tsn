// 计算平方根（牛顿法）
function sqrt(n) {
    if (n == 0) {
        return 0;
    }
    
    let x = n;
    let i = 0;
    while (i < 20) {
        x = (x + n / x) / 2;
        i = i + 1;
    }
    return x;
}

// 计算自然对数（泰勒级数）
function ln(x) {
    // 简化版本：只处理接近 1 的情况
    let result = 0;
    let term = x - 1;
    let i = 1;
    while (i < 20) {
        let sign = i % 2 == 1 ? 1 : 0 - 1;
        result = result + sign * term / i;
        term = term * (x - 1);
        i = i + 1;
    }
    return result;
}

function main() {
    let root = sqrt(16);
    print(root);
    
    let root2 = sqrt(25);
    print(root2);
    
    let log = ln(2);
    // 由于精度问题，打印整数部分
    print(log);
    
    return 0;
}
