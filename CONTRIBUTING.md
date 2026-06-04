# 贡献指南

感谢你考虑为 ts-native 做贡献！

## 开发环境设置

### 前置要求

- Rust 1.95 或更高版本
- Git
- C 编译器（用于运行时）

### 克隆和构建

```bash
git clone https://github.com/itszzl-sudo/ts-native.git
cd ts-native
cargo build
```

### 运行测试

```bash
# 编译测试文件
cargo run -- test_file.ts

# 运行可执行文件
./a.exe
```

## 如何贡献

### 报告 Bug

1. 检查 [Issues](https://github.com/itszzl-sudo/ts-native/issues) 是否已有相同问题
2. 如果没有，创建新 Issue，包含：
   - 清晰的描述
   - 复现步骤
   - 期望行为
   - 实际行为
   - 环境信息

### 提交功能请求

1. 创建 Issue 描述功能
2. 说明为什么需要这个功能
3. 提供使用示例

### 提交代码

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交改动 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 代码规范

### Rust 代码

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 添加必要的注释
- 遵循 Rust API 指南

### 提交信息

- 使用清晰、描述性的标题
- 第一行不超过 50 个字符
- 如有必要，在正文详细说明
- 使用以下前缀：
  - `feat:` 新功能
  - `fix:` Bug 修复
  - `docs:` 文档更新
  - `refactor:` 重构
  - `test:` 测试相关
  - `chore:` 其他改动

## 项目结构

```
ts-native/
├── src/
│   ├── main.rs          # 主入口
│   ├── ts_parser.rs     # TypeScript 解析器
│   ├── codegen.rs       # 代码生成
│   └── linker.rs        # 链接器
├── runtime_nocrt.c      # C 运行时
├── test_*.ts            # 测试文件
└── README.md            # 文档
```

## 测试指南

### 添加新测试

1. 创建 `test_feature.ts` 文件
2. 编写测试代码
3. 确保测试通过：`cargo run -- test_feature.ts && ./a.exe`

### 测试覆盖

确保新增功能有对应的测试覆盖。

## 发布流程

（仅维护者）

1. 更新 CHANGELOG.md
2. 更新版本号
3. 创建 Git 标签
4. 推送到 GitHub
5. 发布到 crates.io

## 许可证

通过贡献代码，你同意你的代码将根据 MIT 许可证授权。

## 联系方式

- GitHub Issues: 用于 Bug 报告和功能请求
- Pull Requests: 用于代码贡献

感谢你的贡献！
