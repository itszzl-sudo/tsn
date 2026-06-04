// 二分查找
function binary_search(arr, target, low, high) {
    if (low > high) {
        return -1;
    }
    
    let mid = (low + high) / 2;
    let mid_val = arr[mid];
    
    if (mid_val == target) {
        return mid;
    }
    
    if (target < mid_val) {
        return binary_search(arr, target, low, mid - 1);
    }
    
    return binary_search(arr, target, mid + 1, high);
}

// 线性查找
function linear_search(arr, target, n) {
    for (let i = 0; i < n; i = i + 1) {
        if (arr[i] == target) {
            return i;
        }
    }
    return -1;
}

function main() {
    let sorted = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19];
    
    let idx1 = binary_search(sorted, 7, 0, 9);
    print(idx1);
    
    let idx2 = binary_search(sorted, 10, 0, 9);
    print(idx2);
    
    let idx3 = linear_search(sorted, 13, 10);
    print(idx3);
    
    let idx4 = linear_search(sorted, 8, 10);
    print(idx4);
    
    return 0;
}
