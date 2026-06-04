function sum(arr, n) {
    let result = 0;
    for (let i = 0; i < n; i = i + 1) {
        result = result + arr[i];
    }
    return result;
}

function product(arr, n) {
    let result = 1;
    for (let i = 0; i < n; i = i + 1) {
        result = result * arr[i];
    }
    return result;
}

function main() {
    let numbers = [2, 3, 4, 5];
    let s = sum(numbers, 4);
    let p = product(numbers, 4);
    print(s);
    print(p);
    return 0;
}
