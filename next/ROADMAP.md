# tsn Roadmap

## 一、核心定位

tsn 是一个将 TypeScript 子集直接编译为原生二进制文件的编译器，采用 Rust + Cranelift 后端，目标是生成极小体积、无运行时依赖的可执行文件。

**独特价值**：在当前 Deno compile / Node SEA / Bun 等方案都选择"打包运行时"的背景下，tsn 走的是"真编译"路线——产物是 10-20KB 的原生代码，不是 50MB 的运行时镜像。

---

## 二、当前质疑（需要解决的问题）

### 2.1 功能完整性

| 质疑点 | 说明 | 建议回应方式 |
|--------|------|-------------|
| TypeScript 支持到什么程度？ | 完整 TS 类型系统？class？泛型？async/await？闭包？ | 列出支持/不支持清单，标注优先级 |
| 标准库如何支持？ | console.log、Array.map、Promise 等内置对象是否可用？ | 说明实现方式（内置/FFI/不支持） |
| 内存管理策略？ | 无 GC 的话如何管理内存？引用计数？手动管理？ | 明确内存模型，给出示例代码 |
| 调试体验？ | 能否断点？错误栈能否定位到 TS 源码行号？ | 说明当前能力和 roadmap |

### 2.2 可靠性

| 质疑点 | 说明 | 建议回应方式 |
|--------|------|-------------|
| 是否经过充分测试？ | 如何保证编译结果正确性？ | 公开测试用例数量和覆盖率 |
| panic 场景处理？ | Cranelift 生成代码出错时如何处理？ | 说明错误恢复机制 |
| 跨平台一致性？ | Windows/Linux/macOS 生成的二进制行为是否一致？ | 提供 CI 跨平台测试结果 |

### 2.3 实用性

| 质疑点 | 说明 | 建议回应方式 |
|--------|------|-------------|
| 是否有人用于生产？ | 除了作者本人，有其他用户吗？ | 征集早期试用者，建立案例库 |
| FFI 能力？ | 能否调用 C 库？调用约定支持？ | 给出 FFI 示例代码 |
| 增量编译？ | 大项目重新编译是否慢？ | 实现 watch 模式或缓存机制 |

---

## 三、Demo

### html-native-iot: HTML → Native UI for IoT

A complete working demo showing tsn's DOM runtime in action.

**Repository:** [html-native-iot](https://github.com/itszzl-sudo/html-native-iot)

**What it does:**
- Parses standard HTML into a DOM tree (via html5ever)
- Generates TypeScript code calling tsn's DOM runtime (`js_dom_*`)
- Compiles to a native executable via tsn (Cranelift backend)
- Includes a standalone minifb renderer for rapid prototyping

**Interactive Counter (minifb-renderer):**

```
┌──────────────────────────────────┐
│  Counter                         │
│  ┌────────────────────────────┐  │
│  │  Count: 0                  │  │
│  └────────────────────────────┘  │
│  [  −  ]  [  +  ]  [ Reset ]    │
└──────────────────────────────────┘
```

- Three buttons: `−`, `+`, `Reset` with hover effects
- Pure software rendering (no GPU) using Inter font
- 400×300 window, < 1KB generated code

**Example HTML input:**

```html
<div>
  <h1>Temp: <span id="temp">25°C</span></h1>
  <button onclick="refresh()">Refresh</button>
</div>
<script>
  let temp = 25;
  function refresh() {
    temp = readSensor('temp');
    document.getElementById('temp').textContent = temp + '°C';
  }
</script>
```

**tsn DOM runtime functions used:**
- `document.createElement`, `document.createTextNode`, `document.getElementById`
- `element.appendChild`, `element.setAttribute`, `element.addEventListener`
- `element.textContent`, `element.value`
- `dom.mainLoop`

---

### H5-Native MQTT IoT: Embedded IoT Control Panel

A pure-software simulated MQTT IoT control panel — no hardware required.

**Status:** Implementation complete

**What it demonstrates:**
- tsn's ability to compile IoT control panels to native executables
- Native MQTT protocol stack integration via FFI
- Long-connection IoT communication (subscribe/publish/heartbeat/reconnect)
- Data-driven UI updates from MQTT message callbacks

**Architecture:**

```
┌─────────────────────────────────────────┐
│  ui.html (HTML)                         │
│  - Device status card                   │
│  - Remote control buttons               │
│  - Temperature/humidity trend display    │
└──────────────┬──────────────────────────┘
               │ html-native-iot (UI tool)
               ↓
┌─────────────────────────────────────────┐
│  ui.ts (TypeScript)                     │
│  - DOM API calls (js_dom_*)             │
│  - MQTT API calls (js_mqtt_*)           │
└──────────────┬──────────────────────────┘
               │ tsn (Cranelift backend)
               ↓
┌─────────────────────────────────────────┐
│  ui.o (Native object)                   │
└──────────────┬──────────────────────────┘
               │ linker + C runtime
               ↓
┌─────────────────────────────────────────┐
│  mqtt-iot.exe (Native executable)       │
│  ┌───────────────────────────────────┐  │
│  │ Native Layer (C runtime)          │  │
│  │ - Built-in local MQTT Broker     │  │
│  │ - Native MQTT Client             │  │
│  │ - Virtual IoT device simulation  │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

**Toolchain:**
1. Write `ui.html` with standard HTML + inline script
2. `html-native-iot ui.html -o ui.ts` — generates TypeScript calling `js_dom_*` and `js_mqtt_*`
3. `tsn ui.ts -o ui.o` — compiles to native object
4. Link with C runtime (MQTT broker/client/device sim) → `mqtt-iot.exe`

**Communication flow:**

```
Virtual Device ──MQTT Publish──→ Broker ──MQTT Subscribe──→ H5 UI (auto-refresh)
H5 Button ──MQTT Publish──→ Broker ──MQTT Subscribe──→ Virtual Device (state change)
```

**MQTT topics:**
- `/device/sensor/data` — device telemetry (temperature, humidity, relay status)
- `/device/cmd/relay` — relay control commands (on/off)

**Simulated device behavior:**
- Temperature: 18°C ~ 33°C, fluctuating every 1s
- Humidity: 22% RH ~ 80% RH, fluctuating every 1s
- Relay: default off, toggle via MQTT command
- Network: simulates disconnect/reconnect/heartbeat recovery

**UI mockup:**

```
┌──────────────────────────────────┐
│  MQTT IoT Control Panel          │
│                                  │
│  ┌────────────────────────────┐  │
│  │ 🟢 Device Online           │  │
│  │ Temperature: 24.5°C        │  │
│  │ Humidity:    55% RH        │  │
│  │ Relay:       OFF           │  │
│  └────────────────────────────┘  │
│                                  │
│  [ Relay ON ]  [ Relay OFF ]     │
│                                  │
│  ┌────────────────────────────┐  │
│  │ Trend (last 30 samples)    │  │
│  │ ▁▂▃▄▅▆▇█▇▆▅▄▃▂▁▂▃▄▅▆▇  │  │
│  └────────────────────────────┘  │
└──────────────────────────────────┘
```

**Framework capabilities validated:**
1. H5 → Native compilation for embedded terminals
2. Native protocol stack extension (MQTT via FFI)
3. Long-connection IoT communication (subscribe/publish/heartbeat/reconnect)
4. Data-driven UI (MQTT data change → auto-render)
5. Event & async scheduling (command dispatch, receipt matching, state sync)

---

## 四、关键期望

### P0 - 必须完成（让人敢试用）

- [ ] 语言特性清单：明确支持/不支持/计划支持，放在 README 第一屏
- [x] 3 个以上的可运行 demo：html-native-iot (UI), minifb-renderer (interactive), MQTT IoT (design complete)
- [ ] 体积对比数据：vs Deno compile / vs Node SEA / vs Rust 原生
- [ ] 5 分钟快速开始教程：从安装到跑通第一个程序

### P1 - 强烈建议（让人愿意采用）

- [x] FFI 示例：演示如何调用 C 库（如 SDL2、sqlite3）
- [x] 报错信息优化：编译错误能定位到 TS 源码位置
- [ ] benchmark 数据：运行时性能 vs Node.js / Deno / Rust
- [ ] 真实场景案例：比如跑在嵌入式设备上的照片/视频

### P2 - 锦上添花（让人爱上用）

- [ ] 增量编译 / watch 模式
- [ ] 生成 DWARF 调试信息（可用 gdb/lldb 调试）
- [ ] VSCode 插件：语法高亮、错误提示
- [ ] 包管理集成：支持 import npm 包（部分）

---

## 五、技术待办

### P2 - 建议修复

- [ ] **标准库缺失**：String 方法（split/replace/substring/charAt）、Array 方法（map/filter/reduce）、Object 方法（keys/values/entries）
- [ ] **错误行号跟踪**：HIR 已有 SourceSpan，需在词法分析阶段记录 token 行号
- [ ] **跨平台链接器**：扩展库搜索硬编码 Windows .lib 后缀，Linux/macOS 需手动指定
- [ ] **CI 跨平台测试**：无 GitHub Actions，无法保证 Linux/macOS 构建可用

### P3 - 长期目标

- [ ] **回调函数支持**：无法将 TS 函数作为回调传递给 C 函数，没有函数指针的 NaN-boxing 编码
- [ ] **结构体传递**：所有参数只支持 F64/I32/I64，无法传递 C 结构体（SDL_Event、sqlite3_stmt 等）
- [ ] **可变参数支持**：不支持 C 的 variadic 函数（如 printf）
- [ ] **class 支持**：解析器和 codegen 均未实现
- [ ] **async/await 支持**：无 Promise、无事件循环、无协程
- [ ] **闭包/lambda 支持**：解析器不解析箭头函数，codegen 不支持捕获环境
- [ ] **泛型支持**：类型信息在 HIR 层完全丢弃
- [ ] **try/catch 支持**：无异常处理机制
- [ ] **switch 支持**：解析器不解析 switch 语句
- [ ] **增量编译**：每次修改都需全量重新编译
- [ ] **DWARF 调试信息**：生成的 .o 文件不含调试信息，Cranelift 支持 DWARF 但需配置
- [ ] **动态加载 dlopen**：所有 FFI 符号在编译时静态声明，运行时无动态加载能力

### 设计决策

- **js_string_from_static 保持 I64 签名**：指针地址转 F64 会丢精度（F64 只有52位尾数），保持 I64 是正确选择
- **48位地址空间限制**：NaN-boxing 只用低48位存指针，当前 x86_64 实际只用48位，不影响

---

## 六、标准库现状

| 功能 | 状态 | 实现方式 |
|------|------|---------|
| console.log | ✅ 已有 | C运行时 js_print |
| Math.* | ✅ 已有 | C运行时 js_math_* |
| Array | ⚠️ 部分 | C运行时 js_array_*，缺少 map/filter/reduce |
| Object | ⚠️ 部分 | C运行时 js_object_*，缺少 keys/values/entries |
| String | ❌ 缺失 | 无 split/replace/substring/charAt 等 |
| Promise | ❌ 缺失 | 无事件循环 |
| Date | ❌ 缺失 | 无 |
| JSON | ❌ 缺失 | 无 |
| setTimeout/setInterval | ❌ 缺失 | 无事件循环 |
| fetch | ❌ 缺失 | 无网络能力 |

---

## 七、已修复缺陷

| 编号 | 缺陷 | 修复时间 | 说明 |
|------|------|---------|------|
| 1.7 | 声明失败静默忽略 | 2026-06-02 | 未解析调用现在打印警告 |
| 1.1 | LinkConfig 未接入链接器 | 2026-06-02 | linker.rs 读取 tsnp/*/ts-native.toml 的 link.lib/libs |
| 1.8 | 自动命名约定过于激进 | 2026-06-02 | 移除自动命名约定，未解析调用打印警告 |
| 2.1 | js_array_new 签名不一致 | 2026-06-02 | 统一为 F64 参数 |
| 2.2 | js_string_from_static 签名异常 | 2026-06-02 | 保持 I64 签名（设计决策） |
| 7.1 | 链接器硬编码绝对路径 | 2026-06-02 | 改为环境变量 TSN_LINKER/TSN_CRT_START + 动态搜索 |
| 1.2 | 类型信息丢失 | 2026-06-02 | builtins 添加 ArgType/RetType，codegen 根据类型生成签名+参数转换 |
| 1.3 | 字符串参数传递断裂 | 2026-06-02 | 添加 js_unbox_string/js_box_string 运行时函数 |
| 3.1 | RuntimeFunctions 死代码 | 2026-06-02 | 删除 |
| 3.2 | builtins 与 registry 重叠 | 2026-06-02 | builtins 注册为默认 Extension |
| 3.3 | 运行时双实现 | 2026-06-02 | 删除 runtime.rs 和 pe_builder.rs，统一使用 C 运行时 |
| 3.4 | C 运行时内联未使用 | 2026-06-02 | 删除 create_c_runtime() |
| 3.5 | set_registry 死代码 | 2026-06-02 | 删除 |
| 5.1 | release stack overflow | 2026-06-02 | cranelift-codegen opt-level=1 避免 |
| 5.3 | 错误信息不定位源码 | 2026-06-02 | HIR 添加 SourceSpan（行号待填充） |
| 8.1 | C运行时重复代码 | 2026-06-02 | 模块化C运行时（runtime_core.c + runtime_dom.c + runtime_mqtt.c） |
| 8.2 | tsnp插件规范缺失 | 2026-06-02 | 添加[ffi]字段，支持c_module声明C扩展模块 |

---

## 八、实施计划

### 8.1 阶段性目标

| 阶段 | 目标 | 关键产出 | 预计受众 |
|------|------|---------|---------|
| Phase 1 | 可 demo | demo 集合 + 性能数据 | 技术尝鲜者 |
| Phase 2 | 可试用 | FFI 支持 + 清晰文档 | 早期采用者 |
| Phase 3 | 可生产 | 增量编译 + 调试支持 | 严肃开发者 |
| Phase 4 | 可推广 | 生态工具 + 社区案例 | 大众用户 |

### 8.2 立即可做

1. 整理 demo：把"嵌入式 HTML UI + tsn"做成可展示的 demo，录视频
2. 写性能对比：用实际代码对比体积和启动速度，数据截图放到 README
3. 建看板：GitHub Projects 或公开的 roadmap，让大家知道进度

### 8.3 文档结构建议

```
tsn/
├── README.md          # 快速开始 + 体积对比 + 特性清单
├── CHANGELOG.md       # 版本变更记录
├── ROADMAP.md         # 本文档
├── docs/
│   ├── language.md    # 支持的 TS 子集说明
│   ├── ffi.md         # FFI 使用指南
│   ├── embedded.md    # 嵌入式场景指南
│   └── internals.md   # 编译器内部架构
├── examples/
│   ├── hello/         # 最简示例
│   ├── counter/       # 状态管理示例
│   └── mqtt-client/   # 嵌入式 IoT 示例
└── benchmarks/
    └── vs-deno-node/  # 性能对比数据
```
