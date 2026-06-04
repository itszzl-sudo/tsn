function swap(arr, i, j) {
    let temp = arr[i];
    arr[i] = arr[j];
    arr[j] = temp;
}

function bubble_sort(arr, n) {
    for (let i = 0; i < n - 1; i = i + 1) {
        for (let j = 0; j < n - i - 1; j = j + 1) {
            if (arr[j] > arr[j + 1]) {
                swap(arr, j, j + 1);
            }
        }
    }
}

function main() {
    let arr = [64, 34, 25, 12, 22, 11, 90];
    
    print(arr[0]);
    print(arr[1]);
    print(arr[2]);
    print(arr[3]);
    print(arr[4]);
    print(arr[5]);
    print(arr[6]);
    
    bubble_sort(arr, 7);
    
    print(arr[0]);
    print(arr[1]);
    print(arr[2]);
    print(arr[3]);
    print(arr[4]);
    print(arr[5]);
    print(arr[6]);
    
    return 0;
}
