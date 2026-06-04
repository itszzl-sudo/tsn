// 实现数组的 map 操作
function map_double(arr, n) {
    let result = [];
    for (let i = 0; i < n; i = i + 1) {
        let val = arr[i] * 2;
        result[i] = val;
    }
    return result;
}

// 实现数组的 filter 操作（保留偶数）
function filter_even(arr, n) {
    let result = [];
    let count = 0;
    for (let i = 0; i < n; i = i + 1) {
        let val = arr[i];
        let is_even = (val / 2) * 2 == val;
        if (is_even) {
            result[count] = val;
            count = count + 1;
        }
    }
    return result;
}

function main() {
    let nums = [1, 2, 3, 4, 5];
    
    let doubled = map_double(nums, 5);
    print(doubled[0]);
    print(doubled[1]);
    print(doubled[2]);
    print(doubled[3]);
    print(doubled[4]);
    
    let filtered = filter_even(nums, 5);
    print(filtered[0]);
    print(filtered[1]);
    
    return 0;
}
