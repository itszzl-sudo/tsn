function is_even(n) {
    if (n == 0) {
        return 1;
    }
    return is_odd(n - 1);
}

function is_odd(n) {
    if (n == 0) {
        return 0;
    }
    return is_even(n - 1);
}

function power(base, exp) {
    if (exp == 0) {
        return 1;
    }
    return base * power(base, exp - 1);
}

function gcd(a, b) {
    if (b == 0) {
        return a;
    }
    return gcd(b, a - b * (a / b));
}

function main() {
    print(is_even(4));
    print(is_odd(5));
    print(power(2, 10));
    print(gcd(48, 18));
    
    let arr = [10, 20, 30, 40, 50];
    let sum = 0;
    for (let i = 0; i < 5; i = i + 1) {
        sum = sum + arr[i];
    }
    print(sum);
    
    let obj = { x: 100, y: 200 };
    print(obj.x + obj.y);
    
    return 0;
}
