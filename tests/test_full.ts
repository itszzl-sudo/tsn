function main() {
    let name = "Test";
    let nums = [1, 2, 3];
    let config = { x: 10, y: 20 };
    
    print(name);
    print(nums[0]);
    print(nums[1]);
    print(nums[2]);
    print(config.x);
    print(config.y);
    
    nums[0] = 100;
    config.x = 200;
    
    print(nums[0]);
    print(config.x);
    
    return 0;
}
