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

function main() {
    print(abs(-5));
    print(abs(10));
    print(max(3, 7));
    print(min(3, 7));
    
    let arr = [-3, 5, -2, 8, -1];
    let max_val = arr[0];
    let min_val = arr[0];
    
    for (let i = 1; i < 5; i = i + 1) {
        max_val = max(max_val, arr[i]);
        min_val = min(min_val, arr[i]);
    }
    
    print(max_val);
    print(min_val);
    
    return 0;
}
