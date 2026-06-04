// 测试复杂表达式和运算符优先级
function main() {
    let a = 2 + 3 * 4;
    let b = (2 + 3) * 4;
    let c = 10 - 2 - 3;
    let d = 100 / 10 / 2;
    
    print(a);
    print(b);
    print(c);
    print(d);
    
    // 测试复杂比较
    let e = 5 < 10 && 10 > 5;
    let f = 5 > 10 || 10 > 5;
    let g = !(5 > 10);
    
    print(e);
    print(f);
    print(g);
    
    return 0;
}
