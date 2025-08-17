# 贡献指南

欢迎您为 RTSP Bruter 项目贡献代码！本指南将帮助您了解如何参与项目开发。

## 项目概述
RTSP Bruter 是一款针对网络摄像头RTSP视频流认证凭证的暴力枚举工具，使用Rust语言开发，基于Tokio异步运行时。

## 贡献方式
您可以通过以下方式为项目做出贡献：
1. 报告Bug
2. 提出功能请求
3. 提交代码改进
4. 完善文档
5. 参与讨论

## 开发环境设置
### 前提条件
- 安装 [Rust](https://www.rust-lang.org/tools/install)
- 安装 [Git](https://git-scm.com/downloads)

### 步骤
1. Fork 本仓库
2. 克隆您的Fork到本地：
   ```bash
   git clone https://github.com/您的用户名/rust-rtsp-bruter.git
   cd rust-rtsp-bruter
   ```
3. 安装依赖：
   ```bash
   cargo build
   ```
4. 运行测试：
   ```bash
   cargo test
   ```

## 代码风格指南
- 遵循 [Rust 官方风格指南](https://doc.rust-lang.org/1.0.0/style/)
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 保持代码简洁、可读性高
- 添加适当的注释，特别是复杂逻辑部分

## 提交代码流程
1. 创建新的分支：
   ```bash
   git checkout -b feature/您的功能名称
   ```
2. 编写代码并提交：
   ```bash
   git add .
   git commit -m "描述您的变更"
   ```
3. 推送到您的Fork：
   ```bash
   git push origin feature/您的功能名称
   ```
4. 提交Pull Request到主仓库

## 提交Pull Request的要求
- 确保代码通过所有测试
- 提供清晰的变更描述
- 保持PR专注于单一功能或修复
- 遵循项目的代码风格
- 更新相关文档（如有必要）

## 报告问题
如果您发现Bug或有功能请求，请在GitHub Issues中提交，并提供以下信息：
- 问题描述
- 复现步骤
- 预期行为
- 实际行为
- 环境信息（如操作系统、Rust版本等）

## 行为规范
请遵守[贡献者公约](https://www.contributor-covenant.org/version/2/1/code_of_conduct/)
，尊重他人，保持友好和专业的沟通。

## 联系我们
如有任何问题，可以通过GitHub Issues或Discussions与我们联系。

感谢您的贡献！