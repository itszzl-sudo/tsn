# ts-native 开发日志

## 项目概述

TypeScript → Native 编译器，基于 PerryTS 架构。

```
TypeScript
    ↓ ts_parser (tokenizer + parser)
   HIR
    ↓ codegen (Cranelift IR)
   .o 文件
    ↓ 链接器
   .exe
```

## 已实现功能

### 1. 语法支持 ✅

**声明：**
- `function` 函数声明
- `let` / `const` 变量声明

**控制流：**
- `if / else` 条件语句
- `while` 循环
- `for` 循环
- `return` 返回语句
- `{ }` 代码块

**运算符：**
- 算术：`+ - * / %`
- 比较：`== != < <= > >=`
- 逻辑：`&& || !`
- 赋值：`=`

**表达式：**
- 数字字面量
- 字符串字面量
- 布尔值 (true/false)
- null / undefined
- 标识符
- 函数调用 `foo(a, b)`
- 数组 `[1, 2, 3]`
- 括号表达式

**其他：**
- 单行注释 `//`
- 多行注释 `/* */`

### 2. 运行时系统 ✅

**数据结构：**
```rust
JsString  - 动态字符串，支持 hash
JsArray   - 动态数组，自动扩容
JsObject  - 键值对对象
```

**NaN-boxing 值表示：**
```
Normal f64:  标准 IEEE 754
Pointer:     0x7FFD | ptr (48 bits)
String:      0x7FFC | ptr
Array:       0x7FFB | ptr
Object:      0x7FFA | ptr
Integer:     0x7FFE | i32
Undefined:   0x7FFF_8000_0000_0001
Null:        0x7FFF_8000_0000_0002
True:        0x7FFF_0000_0000_0001
False:       0x7FFF_0000_0000_0000
```

### 3. 编译器 ✅

**Parser (ts_parser.rs):**
- 完整 tokenizer
- 递归下降解析
- 运算符优先级正确
- 错误恢复

**HIR (codegen.rs):**
- 类型安全的中间表示
- 支持所有语法结构

**CodeGen:**
- Cranelift IR 生成
- SSA 形式
- 块密封正确
- 控制流图正确

## 测试案例

### test_simple.ts (192 bytes)
```typescript
function add(a, b) {
    return a + b;
}
function main() {
    return add(3, 4);
}
```

### test.ts (396 bytes)
```typescript
function add(a, b) {
    return a + b;
}

function factorial(n) {
    let result = 1;
    let i = 1;
    while (i <= n) {
        result = result * i;
        i = i + 1;
    }
    return result;
}

function main() {
    return factorial(5);
}
```

### 内置测试 (280 bytes)
```rust
// 计算 sum(1..10) = 55
let x = 10;
let sum = 0;
while (x > 0) {
    sum = sum + x;
    x = x - 1;
}
return sum;
```

## 项目结构

```
ts-native/
├── Cargo.toml
├── README.md
├── DEVLOG.md (本文件)
└── src/
    ├── main.rs          - 入口
    ├── codegen.rs       - HIR + Cranelift 生成
    ├── ts_parser.rs     - TypeScript 解析器
    └── runtime.rs       - 运行时系统
```

## 下一步

### 短期
- [ ] 链接器集成 (.o → .exe)
- [ ] 函数调用运行时实现
- [ ] 字符串操作
- [ ] 数组操作

### 中期
- [ ] 类型推断
- [ ] 优化 pass
- [ ] 异常处理
- [ ] 垃圾回收

### 长期
- [ ] JSX 支持
- [ ] async/await
- [ ] 模块系统
- [ ] 标准库

## 性能

生成代码质量：
- add: 直接寄存器操作
- while: 紧凑循环
- 无冗余指令

目标文件大小：
- 简单函数: ~200 bytes
- 复杂函数: ~400 bytes

## 参考

- PerryTS: https://github.com/PerryTS/perry
- Cranelift: https://github.com/bytecodealliance/wasmtime/tree/main/cranelift
- NaN-boxing: JavaScriptCore / LuaJIT
