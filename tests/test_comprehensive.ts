function square(x) {
    return x * x;
}

function sum_array(arr, n) {
    let sum = 0;
    let i = 0;
    while (i < n) {
        sum = sum + arr[i];
        i = i + 1;
    }
    return sum;
}

function main() {
    let nums = [1, 2, 3, 4, 5];
    let result = sum_array(nums, 5);
    print(result);
    
    let sq = square(7);
    print(sq);
    
    let obj = { x: 10, y: 20 };
    print(obj.x + obj.y);
    
    return 0;
}
