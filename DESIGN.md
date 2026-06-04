# tsn 工具链设计文档

## 整体架构

```
tsn (编译器)
  ↓ 编译 TypeScript → 原生 exe
tsnp (插件生成器)
  ↓ 分析 Rust crate → 生成配置
cargo-tsn (项目管理器)
  ↓ 创建项目、添加依赖、添加 FFI 函数
```

## 工具职责

| 工具 | 职责 | 命令示例 |
|------|------|----------|
| **tsn** | 编译 TypeScript | `tsn main.ts` |
| **tsnp** | 生成插件配置 | `tsnp gen regex` |
| **cargo-tsn** | 项目管理 | `cargo tsn new my-project` |

## cargo-tsn 命令

### cargo tsn new <name>

创建 tsn 项目。

**生成目录结构：**
```
my-project/
├── Cargo.toml       # Rust 配置
├── src/
│   └── lib.rs       # FFI 函数（用户编写）
├── main.ts          # TypeScript 代码
└── tsnp/            # 插件配置目录
```

**Cargo.toml 内容：**
```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
```

**src/lib.rs 内容：**
```rust
// Export FFI functions here
// Example:
// #[no_mangle]
// pub extern "C" fn my_func() -> i32 { 0 }
```

### cargo tsn add <crate>

添加 Rust crate 依赖并生成插件配置。

**流程：**
1. `cargo add <crate>` - 下载依赖
2. `tsnp gen <crate>` - 生成插件配置
3. 放到 `tsnp/<crate>/`

**输出：**
```
tsnp/<crate>/
├── ts-native.toml    # 函数映射配置
├── index.d.ts        # TypeScript 类型定义
└── README.md         # 使用说明
```

**注意：** 大部分 crate 没有 FFI 函数，生成的是空模板。用户需要：
- 在 `src/lib.rs` 中包装 FFI
- 编辑 `tsnp/<crate>/ts-native.toml`

### cargo tsn func

交互式添加 FFI 函数。

**流程：**
```
> cargo tsn func

Select crate:
[1] regex      (tsnp/regex/)
[2] math       (tsnp/math/)
[q] Quit
Select: 1

regex selected.

Function name (or 'q'): add
Parameters (e.g., 'a: i32, b: i32'): a: i32, b: i32
Return type (e.g., 'i32'): i32

✅ Added to src/lib.rs
✅ Updated tsnp/regex/ts-native.toml

Function name (or 'q'): multiply
Parameters (e.g., 'a: i32, b: i32'): a: i32, b: i32
Return type (e.g., 'i32'): i32

✅ Added to src/lib.rs
✅ Updated tsnp/regex/ts-native.toml

Function name (or 'q'): q

Select crate:
[1] regex      (tsnp/regex/)
[2] math       (tsnp/math/)
[q] Quit
Select: 2

math selected.

Function name (or 'q'): square
Parameters (e.g., 'n: i32'): n: i32
Return type (e.g., 'i32'): i32

✅ Added to src/lib.rs
✅ Updated tsnp/math/ts-native.toml

Function name (or 'q'): q

Select crate:
[1] regex      (tsnp/regex/)
[2] math       (tsnp/math/)
[q] Quit
Select: q

Done. 3 FFI function(s) added.
```

**交互逻辑：**
- 输入 `q` 返回 crate 选择列表
- 在 crate 选择列表输入 `q` 退出命令
- 可切换不同 crate 添加函数

## tsnp 命令

### tsnp gen <name>

分析 Rust crate，生成插件配置。

**源码来源（三选一）：**
```
[1] Local path     # 已有源码
[2] GitHub tarball # 自动下载
[3] Search API     # 只下载 FFI 文件（需 token）
```

**输出：**
```
tsnp/<name>/
├── ts-native.toml
├── index.d.ts
└── README.md
```

### tsnp new <name>

创建空插件模板。

**用途：** 不分析源码，手动配置。

## tsn 命令

### tsn main.ts

编译 TypeScript 为原生可执行文件。

**默认输出：**
- `<input>.o` - 目标文件（与输入同目录）
- `<input>.exe` - 可执行文件（与输入同目录）

**扩展发现：**
- 检查 `<input>.ts.toml` 依赖声明
- 如存在 → 只加载声明的 tsnp 扩展
- 如不存在 → 扫描 `./tsnp/` 目录加载全部

## ts-native.toml 格式

```toml
[package]
name = "tsnp-regex"
version = "0.1.0"
description = "Regex extension for tsn"

[ffi]
c_module = "runtime/runtime_regex.c"

[functions]
"add" = { args = ["number", "number"], ret = "number", impl_name = "add" }
"multiply" = { args = ["number", "number"], ret = "number", impl_name = "multiply" }

[link]
libs = ["regex"]
flags = []
```

**字段说明：**
- `[ffi]` - C扩展模块配置（可选）
  - `c_module` - C源文件路径（相对于tsn根目录）
- `[functions]` - FFI函数映射
- `[link]` - 链接配置
  - `libs` - 需要链接的库列表（数组格式）
  - `flags` - 链接标志（数组格式）

## 类型映射

| Rust | TypeScript |
|------|------------|
| i8, u8, i16, u16, i32, u32, i64, u64, isize, usize | number |
| f32, f64 | number |
| *const c_char, *mut c_char | string |
| *const T, *mut T | number (指针) |
| &T, &mut T | number (指针) |
| () | void |

## 完整工作流

```bash
# 1. 创建项目
cargo tsn new my-project
cd my-project

# 2. 添加依赖
cargo tsn add regex
cargo tsn add serde_json

# 3. 交互式添加 FFI（或手动编写）
cargo tsn func

# 4. 编译
tsn main.ts

# 5. 运行
./a.exe
```

## 项目结构示例

```
my-project/
├── Cargo.toml           # Rust 配置
├── Cargo.lock
├── src/
│   └── lib.rs           # FFI 函数
├── main.ts              # TypeScript 代码
├── tsnp/                # 插件配置
│   ├── regex/
│   │   ├── ts-native.toml
│   │   ├── index.d.ts
│   │   └── README.md
│   └── serde_json/
│       ├── ts-native.toml
│       ├── index.d.ts
│       └── README.md
├── main.o                  # 目标文件
└── main.exe                # 可执行文件
```

## 安装

```bash
cargo install tsn
cargo install tsnp
cargo install cargo-tsn
```

## 版本

| 工具 | 版本 |
|------|------|
| tsn | v0.1.0 |
| tsnp | v0.1.0 |
| cargo-tsn | v0.1.0 |

## 许可证

MIT
