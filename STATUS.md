---
## Goal

用户正在开发 **ts-native 编译器**：将 TypeScript 直接编译为 Native 可执行文件。

**主要目标**：
```
TypeScript → ts_parser → HIR → Cranelift → .o 文件 → .exe
```

**当前任务**：
基础功能已实现，下一步是链接器集成和运行时函数支持。

---

## Instructions

- 使用中文交互
- 安静编码，不询问用户
- 不要做删除等需要授权的操作
- 偏好简洁直接的输出
- 使用 snake_case 命名
- 用户偏好自主推进，使用简短指令如'继续'

---

## Discoveries

### 1. Cranelift 0.93 API 关键点
- `FloatCC` 而非 `FloatCmp`
- `jump(block, &[])` 需要空数组参数
- `brif(cond, then_block, &[], else_block, &[])` 需要 4 个参数
- `module.declare_func_in_func(func_id, &mut builder.func)` 正确获取函数引用
- `builder.ins().call(func_ref, &args)` 调用函数

### 2. 两遍编译架构
- 第一遍：收集所有函数签名和字符串常量
- 第二遍：编译函数体，可以引用已收集的 FuncId

### 3. NaN-boxing 值表示
- 所有 JS 值存为 f64（8字节）
- STRING_TAG = 0x7FFC_0000_0000_0000
- ARRAY_TAG = 0x7FFB_0000_0000_0000
- OBJECT_TAG = 0x7FFA_0000_0000_0000
- UNDEFINED = 0x7FFF_8000_0000_0001
- NULL = 0x7FFF_8000_0000_0002

---

## Accomplished

### ✅ 已完成：

1. **完整 TypeScript 解析器** (ts_parser.rs)
   - 运算符优先级正确
   - 控制流语句
   - 函数调用解析
   - 注释支持
   - 数组字面量 `[1, 2, 3]`
   - 对象字面量 `{ x: 10 }` (基础)

2. **HIR 中间表示** (codegen.rs)
   - 所有语法结构
   - 类型安全

3. **Cranelift 代码生成** (codegen.rs)
   - if/else 条件
   - while/for 循环
   - 二元运算
   - NaN-boxing 值表示
   - 函数声明
   - 函数调用（已修复）
   - 字符串字面量
   - 数组字面量
   - 对象字面量
   - 数组索引访问 `arr[i]`
   - 对象属性访问 `obj.prop`

4. **字符串/数组/对象支持**
   - 字符串哈希值存储
   - 数组长度存储
   - 对象属性数量存储

5. **运行时系统** (runtime.rs)
   - JsString 字符串
   - JsArray 动态数组
   - JsObject 对象

6. **链接器支持** (linker.rs)
   - Windows/Linux/macOS
   - GCC/Clang 集成

7. **测试验证**
   - `--test` 测试通过（280 bytes）
   - 函数调用测试通过
   - 字符串/数组测试通过

---

## Relevant files / directories

### ts-native 项目根目录
```
E:/Administrator/Documents/codebuddy-projects/ts-native/
```

### 核心文件：

**src/codegen.rs** - 代码生成器
- 第 6-12 行：NaN-boxing 常量
- 第 59-63 行：CodeGen 结构体
- 第 89-102 行：第一遍编译
- 第 104-116 行：字符串数据段
- 第 227-280 行：compile_function 函数
- 第 546-638 行：表达式编译（Call, Array, Index, Property）

**src/ts_parser.rs** - TypeScript 解析器
- 完整的 tokenizer + 递归下降解析器
- 支持所有语法

**src/runtime.rs** - 运行时
- JsString, JsArray, JsObject
- NaN-boxing 函数

**src/linker.rs** - 链接器
- Windows/Linux/macOS 支持

---

## 下一步任务

### 高优先级：
1. **链接器集成**
   - 安装 GCC/Clang
   - 测试生成可执行文件
   - 添加 C 运行时

2. **运行时函数**
   - console.log/print
   - 内存分配 malloc
   - 字符串操作

3. **完善数组/对象**
   - 数组元素存储
   - 对象属性存储
   - 索引越界检查

### 中优先级：
4. **标准库**
   - Math 函数
   - 数组方法
   - 字符串方法

5. **错误处理**
   - 编译错误
   - 运行时错误

---
