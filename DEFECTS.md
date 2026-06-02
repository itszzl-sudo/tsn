# tsn 已知缺陷与问题清单

> 最后更新: 2026-06-02

## 已修复缺陷

| 编号 | 缺陷 | 修复时间 | 说明 |
|------|------|---------|------|
| 1.7 | 声明失败静默忽略 | 2026-06-02 | 未解析调用现在打印警告 |
| 1.1 | LinkConfig 未接入链接器 | 2026-06-02 | linker.rs 读取 tsnp/*/ts-native.toml 的 link.lib/libs |
| 1.8 | 自动命名约定过于激进 | 2026-06-02 | 移除自动命名约定，未解析调用打印警告 |
| 2.1 | js_array_new 签名不一致 | 2026-06-02 | 统一为 F64 参数 |
| 2.2 | js_string_from_static 签名异常 | 2026-06-02 | 保持 I64 签名（指针地址不适合转 F64，会丢精度），记录为设计决策 |
| 7.1 | 链接器硬编码绝对路径 | 2026-06-02 | 改为环境变量 TSN_LINKER/TSN_CRT_START + 动态搜索 |
| - | main.rs flag 解析简陋 | 2026-06-02 | 重写为位置无关的 flag 解析，输出文件从输入推导 |
| - | runtime.rs UB 问题 | 2026-06-02 | alloc 空指针检查，grow/push/set 返回 bool，Layout 用 ok()? 替代 unwrap() |
| - | linker.rs 重复代码 | 2026-06-02 | 移除 find_crt_start 中残留的重复行 |

---

## 一、FFI 机制缺陷

### 1.2 类型信息丢失

- **位置**: `codegen.rs` 所有 `declare_function` 调用
- **问题**: `ArgType`/`RetType` 仅用于清单描述，codegen 中所有参数一律按 F64 处理，忽略实际类型
- **影响**: 无法正确传递 I32/I64/指针类型参数给 C 函数，ABI 不匹配时静默崩溃
- **修复难度**: 中

### 1.3 字符串参数传递断裂

- **位置**: `codegen.rs` Call 节点处理
- **问题**: C 库期望 `const char*`，tsn 传的是 NaN-boxed F64，无运行时拆箱转换层
- **影响**: 无法直接调用接受字符串参数的 C 函数
- **修复难度**: 中

### 1.4 回调函数不支持

- **问题**: 无法将 TS 函数作为回调传递给 C 函数，没有函数指针的 NaN-boxing 编码
- **影响**: 无法使用 sqlite3_exec、SDL_AddEventWatch 等需要回调的 C API
- **修复难度**: 高

### 1.5 结构体传递不支持

- **问题**: 所有参数只支持 F64（及极少数 I32/I64），无法传递 C 结构体
- **影响**: 无法使用 SDL_Event、sqlite3_stmt 等结构体参数的 C API
- **修复难度**: 高

### 1.6 可变参数不支持

- **问题**: 不支持 C 的 variadic 函数（如 printf）
- **影响**: 无法直接调用 printf 等可变参数 C 函数
- **修复难度**: 中

### 1.9 48位地址空间限制

- **问题**: NaN-boxing 只用低48位存指针，不支持超过48位地址的平台
- **影响**: 在使用超过256TB虚拟地址的平台上指针截断
- **修复难度**: 低（当前x86_64实际只用48位）

---

## 二、调用约定（设计决策）

### 2.2 js_string_from_static 保持 I64 签名

- **位置**: `codegen.rs` 字符串字面量处理
- **说明**: 签名为 `(I64) → F64`，接收数据段地址。指针地址转 F64 会丢精度（F64 只有52位尾数），因此保持 I64 是正确选择
- **影响**: 与其他 FFI 函数的调用约定不一致，但这是必要的例外

---

## 三、架构设计问题

### 3.1 RuntimeFunctions 死代码

- **位置**: `codegen.rs` 第986-1004行
- **问题**: `declare_runtime_functions` 生成了 `RuntimeFunctions` 结构，但整个编译流程中 `_runtime_funcs` 从未被使用
- **影响**: 死代码，增加维护负担
- **修复难度**: 低（删除即可）

### 3.2 builtins 与 registry 职责重叠

- **位置**: `builtins.rs` + `extension.rs`
- **问题**: `builtins.rs` 中的映射完全可以用 `ExtensionRegistry` + ts-native.toml 替代，当前是两套并行的映射机制
- **影响**: 新增内置函数需同时改两个地方，容易遗漏
- **修复难度**: 中（将 builtins 迁移为默认 Extension）

### 3.3 运行时双实现

- **位置**: `runtime.rs`（Rust）+ `linker.rs::create_c_runtime()`（C）+ 外部 `runtime_nocrt.o`
- **问题**: 三套运行时实现，`runtime.rs` 和 `create_c_runtime()` 都标记为 `#[allow(dead_code)]` 未使用
- **影响**: 代码冗余，新人困惑该用哪个
- **修复难度**: 中（统一为一套）

### 3.4 C 运行时内联未使用

- **位置**: `linker.rs` 第257行 `create_c_runtime()`
- **问题**: 返回 C 源码字符串，但标记为 `#[allow(dead_code)]`，实际未使用
- **影响**: 死代码
- **修复难度**: 低

### 3.5 set_registry 死代码

- **位置**: `codegen.rs` 第80行
- **问题**: `CodeGen::set_registry()` 方法未使用（用 `with_registry` 替代）
- **影响**: 编译警告
- **修复难度**: 低（删除即可）

---

## 四、语言特性缺失

### 4.1 动态加载 / dlopen 不支持

- **问题**: 所有 FFI 符号在编译时静态声明为 `Linkage::Import`，运行时无动态加载能力
- **影响**: 无法在运行时按需加载共享库
- **修复难度**: 高

### 4.2 class 不支持

- **问题**: 解析器和 codegen 均未实现 class 语法
- **影响**: 无法编译使用 class 的 TypeScript 代码
- **修复难度**: 高

### 4.3 async/await 不支持

- **问题**: 无 Promise、无事件循环、无协程
- **影响**: 无法编译异步代码
- **修复难度**: 高

### 4.4 闭包/lambda 不支持

- **问题**: 解析器不解析箭头函数，codegen 不支持捕获环境
- **影响**: 无法使用 `() => {}` 或闭包
- **修复难度**: 高

### 4.5 泛型不支持

- **问题**: 类型信息在 HIR 层完全丢弃
- **影响**: 无法编译使用泛型的代码
- **修复难度**: 高（需重新设计类型系统）

### 4.6 try/catch 不支持

- **问题**: 无异常处理机制
- **影响**: 无法编译 try/catch 代码
- **修复难度**: 中

### 4.7 switch 不支持

- **问题**: 解析器不解析 switch 语句
- **影响**: 无法编译 switch/case 代码
- **修复难度**: 低

---

## 五、编译器质量问题

### 5.1 release 构建 stack overflow

- **问题**: `cargo build --release` 时 Cranelift 编译触发 `STATUS_STACK_BUFFER_OVERRUN`
- **影响**: 无法生成 release 优化构建
- **修复难度**: 中（可能需调整 Cranelift 优化参数或 Rust 栈大小）

### 5.2 无增量编译

- **问题**: 每次修改都需全量重新编译
- **影响**: 开发体验差，大项目编译慢
- **修复难度**: 高

### 5.3 错误信息不定位源码

- **问题**: 编译错误只报告 HIR 层位置，无法映射回 TS 源码行号
- **影响**: 调试困难
- **修复难度**: 中（需在解析阶段记录源码位置）

### 5.4 无 DWARF 调试信息

- **问题**: 生成的 .o 文件不含调试信息
- **影响**: 无法用 gdb/lldb 调试生成的代码
- **修复难度**: 高（Cranelift 支持 DWARF 但需配置）

---

## 六、标准库缺失

| 功能 | 状态 | 实现方式 |
|------|------|---------|
| console.log | ✅ 已有 | C运行时 js_print |
| Math.* | ✅ 已有 | C运行时 js_math_* |
| Array | ⚠️ 部分 | C运行时 js_array_*，但缺少 map/filter/reduce |
| Object | ⚠️ 部分 | C运行时 js_object_*，但缺少 keys/values/entries |
| String | ❌ 缺失 | 无 split/replace/substring/charAt 等 |
| Promise | ❌ 缺失 | 无事件循环 |
| Date | ❌ 缺失 | 无 |
| JSON | ❌ 缺失 | 无 |
| setTimeout/setInterval | ❌ 缺失 | 无事件循环 |
| fetch | ❌ 缺失 | 无网络能力 |

---

## 七、跨平台问题

### 7.1 链接器跨平台支持不完整

- **位置**: `linker.rs`
- **问题**: 扩展库搜索硬编码 Windows .lib 后缀，Linux/macOS 需手动指定
- **影响**: Linux/macOS 链接需额外配置
- **修复难度**: 中

### 7.2 无 CI 跨平台测试

- **问题**: 无 GitHub Actions 等跨平台 CI
- **影响**: 无法保证 Linux/macOS 构建可用
- **修复难度**: 低

---

## 八、优先级排序

### P0 - 必须修复（影响正确性）

（全部已修复 ✅）

### P1 - 应当修复（影响可用性）

1. 类型信息丢失（1.2）
2. 字符串参数传递断裂（1.3）
3. 错误信息不定位源码（5.3）

### P2 - 建议修复（影响体验）

4. RuntimeFunctions 死代码（3.1）
5. set_registry 死代码（3.5）
6. builtins 与 registry 重叠（3.2）
7. 运行时双实现（3.3）
8. release 构建 stack overflow（5.1）
9. 标准库缺失（String/Array方法等）

### P3 - 长期目标

10. 回调函数支持（1.4）
11. 结构体传递（1.5）
12. class/async/闭包/泛型（4.2-4.5）
13. 增量编译（5.2）
14. DWARF 调试信息（5.4）
