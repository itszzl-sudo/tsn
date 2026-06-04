# ts-native 使用示例

## 快速开始

### 1. 构建编译器
```bash
cargo build --release
```

### 2. 编译 TypeScript 文件
```bash
# 编译为 .o 目标文件
ts-native input.ts -o output.o

# 或使用构建脚本
./build.sh input.ts      # Linux/macOS
build.bat input.ts       # Windows
```

### 3. 运行测试
```bash
ts-native --test
```

---

## 示例 1: 函数定义和调用

```typescript
// function.ts
function add(a, b) {
    return a + b;
}

function main() {
    return add(3, 5);
}
```

编译：
```bash
ts-native function.ts -o function.o
```

---

## 示例 2: 循环和累加

```typescript
// sum.ts
function sum(n) {
    let result = 0;
    let i = 1;
    
    while (i <= n) {
        result = result + i;
        i = i + 1;
    }
    
    return result;
}

function main() {
    return sum(100);  // 5050
}
```

编译：
```bash
ts-native sum.ts -o sum.o
```

---

## 示例 3: 递归函数

```typescript
// factorial.ts
function factorial(n) {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

function main() {
    return factorial(10);  // 3628800
}
```

编译：
```bash
ts-native factorial.ts -o factorial.o
```

---

## 示例 4: 条件判断

```typescript
// prime.ts
function isPrime(n) {
    if (n <= 1) {
        return 0;
    }
    
    let i = 2;
    while (i * i <= n) {
        if (n % i == 0) {
            return 0;
        }
        i = i + 1;
    }
    
    return 1;
}

function main() {
    return isPrime(97);  // 1 (prime)
}
```

---

## 示例 5: Fibonacci

```typescript
// fibonacci.ts
function fibonacci(n) {
    if (n <= 1) {
        return n;
    }
    
    let a = 0;
    let b = 1;
    let i = 2;
    
    while (i <= n) {
        let temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }
    
    return b;
}

function main() {
    return fibonacci(20);  // 6765
}
```

---

## 示例 6: for 循环

```typescript
// forloop.ts
function main() {
    let sum = 0;
    
    for (let i = 1; i <= 10; i = i + 1) {
        sum = sum + i;
    }
    
    return sum;  // 55
}
```

---

## 示例 7: 嵌套条件

```typescript
// nested.ts
function max(a, b, c) {
    if (a >= b) {
        if (a >= c) {
            return a;
        } else {
            return c;
        }
    } else {
        if (b >= c) {
            return b;
        } else {
            return c;
        }
    }
}

function main() {
    return max(5, 9, 3);  // 9
}
```

---

## 完整程序结构

每个程序必须有一个 `main` 函数：

```typescript
function main() {
    // 你的代码
    return 0;
}
```

---

## 性能优化建议

### 1. 使用 release 构建
```bash
cargo build --release
```

### 2. 启用 LTO (链接时优化)
在 Cargo.toml 中：
```toml
[profile.release]
lto = true
opt-level = 3
```

### 3. 使用内联函数
```typescript
function square(x) {
    return x * x;
}

function main() {
    // square 会被内联
    return square(5) + square(3);
}
```

---

## 限制和已知问题

### 当前不支持
- 类 (class)
- 接口 (interface)
- 泛型 (generics)
- 异步函数 (async/await)
- 模块导入 (import/export)
- try-catch 异常处理
- 字符串模板
- 解构赋值

### 已知限制
- 递归深度受栈大小限制
- 浮点精度为 IEEE 754 double
- 数组操作需要运行时支持

---

## 调试技巧

### 查看生成的 HIR
```bash
RUST_LOG=debug ts-native input.ts
```

### 查看目标文件符号
```bash
nm output.o           # Linux/macOS
dumpbin /symbols output.o  # Windows
```

### 反汇编查看代码
```bash
objdump -d output.o   # Linux/macOS
dumpbin /disasm output.o  # Windows
```

---

## 贡献指南

欢迎提交 PR！

### 开发环境设置
```bash
git clone <repo>
cd ts-native
cargo build
cargo test
```

### 运行测试
```bash
cargo test
cargo run -- --test
```

### 代码风格
```bash
cargo fmt
cargo clippy
```
