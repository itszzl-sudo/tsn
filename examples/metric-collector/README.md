# TS-Native 运维采集完整示例

演示如何使用 TS-Native + FFI 实现完整的监控采集方案。

## 项目组成

```
metric-collector/
├── main.ts            # 完整版采集器（FFI + 循环采集）
├── main-simple.ts     # 精简版采集器（有限循环）
├── server.ts          # 原生HTTP API服务器
├── main.ts.toml       # main.ts 依赖声明
├── main-simple.ts.toml # main-simple.ts 依赖声明
├── server.ts.toml     # server.ts 依赖声明
├── build.bat          # Windows 构建脚本
├── build.sh           # Linux/macOS 构建脚本
└── tsnp/              # FFI 扩展包
    ├── metric-collector/
    │   ├── ts-native.toml
    │   └── index.d.ts
    └── metric-api/
        ├── ts-native.toml
        └── index.d.ts
```

## 快速开始

### 1. 编译

```bash
# 完整版
ts-native main.ts
# → main.exe (同目录下)

# 精简版
ts-native main-simple.ts
# → main-simple.exe

# API服务器
ts-native server.ts
# → server.exe
```

### 2. 运行

```bash
# 完整版（默认无限循环，60秒间隔）
./main.exe

# 精简版（采集3次后退出）
./main-simple.exe

# API服务器
./server.exe
```

### 3. 查看采集日志

```bash
cat metric.log
# [2026-06-03 15:06:37] CPU=2.20% MEM=2.20% REQ=805 RT=0ms
```

## 依赖声明（.ts.toml）

每个 TypeScript 文件通过 `<filename>.ts.toml` 声明所需的 tsnp 扩展：

**main.ts.toml**:
```toml
[dependencies]
tsnp = ["metric-collector"]
```

**server.ts.toml**:
```toml
[dependencies]
tsnp = ["metric-api"]
```

- 如果 `.ts.toml` 存在 → 只加载声明的扩展
- 如果 `.ts.toml` 不存在 → 加载 tsnp/ 下全部扩展

## 工作流程

```
┌──────────────────────────────┐
│  main.ts                     │
│  - 解析命令行参数            │
│  - 循环采集指标              │
│  - FFI: http_get, sleep      │
│  - FFI: file_append, now_ms  │
└───────────┬──────────────────┘
            │
            ↓ HTTP GET / file write
            │
┌──────────────────────────────┐
│  metric.log                  │
│  - 时间戳 + CPU/内存/请求    │
│  - 真实日期格式 (YYYY-MM-DD) │
└──────────────────────────────┘
```

## 性能

| 指标 | 数值 |
|------|------|
| 二进制大小 | ~11-23KB |
| 启动时间 | < 1ms |
| 内存占用 | < 5MB |
| 依赖 | 无（无需 Node.js） |

## 采集器参数

```bash
main.exe [url] [interval] [maxRuns]

参数：
  url      - 采集地址（默认: https://httpbin.org/json）
  interval - 采集间隔秒数（默认: 60）
  maxRuns  - 最大运行次数（默认: 0 = 无限）
```

## 内置功能

| 功能 | 说明 |
|------|------|
| `console.log()` / `print()` | 控制台输出 |
| `Math.random()` / `Math.floor()` | 数学函数 |
| `Date.now()` | 时间戳（毫秒） |
| `JSON.stringify()` / `JSON.parse()` | JSON 序列化 |
| `parseInt()` | 字符串转数字 |
| `toFixed()` | 数字格式化 |
| 模板字符串 | `` `hello ${var}` `` |
| try/catch/finally | 异常处理 |
| for...in / for...of | 迭代 |

## 总结

✅ **原生编译**：TypeScript → 原生二进制（11-23KB）
✅ **FFI 扩展**：真实系统调用（HTTP、文件、sleep）
✅ **零依赖**：无需 Node.js 运行时
✅ **语法检查**：swc 前置检测不支持的语法
✅ **依赖声明**：`.ts.toml` 精确控制扩展加载
✅ **真实应用**：完整可用的监控采集方案
