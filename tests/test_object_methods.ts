// 模拟对象方法
function create_point(x, y) {
    let point = { x: x, y: y };
    return point;
}

function point_add(p1, p2) {
    let result = { x: 0, y: 0 };
    result.x = p1.x + p2.x;
    result.y = p1.y + p2.y;
    return result;
}

function point_distance_squared(p) {
    return p.x * p.x + p.y * p.y;
}

function main() {
    let p1 = create_point(3, 4);
    let p2 = create_point(1, 2);
    
    print(p1.x);
    print(p1.y);
    
    let sum = point_add(p1, p2);
    print(sum.x);
    print(sum.y);
    
    let dist_sq = point_distance_squared(p1);
    print(dist_sq);
    
    return 0;
}
