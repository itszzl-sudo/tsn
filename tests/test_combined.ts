function main() {
    let arr = [1, 2, 3];
    let obj = { x: 10, y: 20 };
    print(arr[0]);
    print(arr[1]);
    print(obj.x);
    print(obj.y);
    obj.x = 100;
    arr[0] = 99;
    print(arr[0]);
    print(obj.x);
    return 0;
}
