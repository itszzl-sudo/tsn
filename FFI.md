# tsn FFI 调用 C 库：现状与支持难度

## 一、当前 FFI 机制

### 调用链路

```
TS源码 print(x) → codegen声明 js_print 为 Linkage::Import → 链接器从 runtime_nocrt.o 解析符号 → exe
```

### 调用约定

所有参数/返回值通过 F64 传递（NaN-boxing编码），C端用 `uint64_t` 接收再 reinterpret：

```c
uint64_t js_add(uint64_t a, uint64_t b) {
    double da = *(double*)&a;
    double db = *(double*)&b;
    double result = da + db;
    return *(uint64_t*)&result;
}
```

### 三种调用路径

| 方式 | 示例 | 状态 |
|------|------|------|
| 内置映射 | `Math.sin(x)` → `js_math_sin` | ✅ 可用 |
| 插件注册 | `tsnp/regex/ts-native.toml` 定义映射 | ⚠️ codegen走通，但链接器未读LinkConfig |
| 自动命名 | `foo.bar(x)` → `js_foo_bar` | ⚠️ 声明生成，但链接时若无实现则失败 |

### FFI 解析优先级

1. 用户定义函数（function_ids）→ 直接调用
2. 硬编码特殊函数（print/console.log/parseInt/Boolean）→ 内联或 js_print
3. builtins 映射表（Math.*/document.*/element.*/browser.*）→ Linkage::Import
4. ExtensionRegistry 插件注册表 → Linkage::Import
5. 自动命名约定（含点号→js_前缀）→ Linkage::Import
6. 兜底 → 返回 UNDEFINED

---

## 二、调用任意 C 库需要补什么

### 2.1 LinkConfig 接入链接器（难度：低）

- **现状**: linker.rs 扩展库搜索路径硬编码，未读 Extension.link.libs
- **需要**: linker 从 ExtensionRegistry 读取 link.libs 配置，加入链接参数
- **工作量**: 约2小时

### 2.2 类型安全 FFI 声明（难度：中）

- **现状**: 所有参数一律按 F64 处理，忽略 ArgType/RetType
- **需要**: 根据 ArgType 生成正确的参数类型（I32/I64/F64/指针）
- **工作量**: 约1天

### 2.3 字符串参数转换（难度：中）

- **现状**: C 库期望 `const char*`，tsn 传的是 NaN-boxed F64
- **需要**: 运行时拆箱层：F64 → 检查 STRING_TAG → 提取指针 → 返回 `const char*`
- **工作量**: 约1天

### 2.4 回调函数支持（难度：高）

- **现状**: 无法将 TS 函数作为回调传递给 C 函数
- **需要**: 函数指针的 NaN-boxing 编码 + 运行时 trampoline 函数
- **工作量**: 约3-5天

### 2.5 结构体传递（难度：高）

- **现状**: 无法传递 C 结构体
- **需要**: TS 侧定义对应内存布局 + 按偏移量读写字段
- **工作量**: 约1-2周

---

## 三、各场景支持难度总结

| 场景 | 难度 | 前置条件 | 预计工作量 |
|------|------|---------|-----------|
| 调用纯数值C函数 | ✅ 已可用 | 写C包装函数遵守NaN-boxing约定 | 0 |
| 调用含字符串参数的C函数 | 中 | 字符串转换层 | 1天 |
| 调用含结构体的C函数 | 高 | 结构体布局定义 | 1-2周 |
| 调用含回调的C函数 | 高 | 函数指针编码 + trampoline | 3-5天 |
| 动态加载共享库 | 高 | dlopen/dlsym 封装 | 1周 |
| 完整C库绑定（如SDL2） | 高 | 以上全部 | 2-3周 |

---

## 四、推荐实施路径

```
Phase 1: LinkConfig接入链接器（2h）→ 插件FFI可自动链接
Phase 2: 类型安全签名（1d）→ 支持I32/I64/指针参数
Phase 3: 字符串转换层（1d）→ 可调用含字符串的C函数
Phase 4: 回调函数支持（3-5d）→ 可调用sqlite3等需回调的库
Phase 5: 结构体传递（1-2w）→ 可调用SDL2等需结构体的库
```

Phase 1-3 完成后，tsn 即可实际调用大多数常用 C 库（sqlite3、libc、数学库等）。