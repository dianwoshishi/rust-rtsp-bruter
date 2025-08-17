# IP端口解析器示例

这个目录包含了一个展示`ip_port_parser`功能的示例程序。

## 示例程序

`ip_port_parser_example.rs` - 展示如何使用`parse_ip_port`函数解析各种IP端口格式，包括：
- 单个IP和端口
- IP范围和端口范围
- 带CIDR掩码的IP
- 复杂的IP模式（带花括号）
- 带CIDR掩码的复杂IP模式

## 如何运行

1. 确保你已经安装了Rust开发环境。如果没有，请访问 https://www.rust-lang.org/ 安装。

2. 在项目根目录下，使用以下命令编译并运行示例：

```bash
cargo run --example ip_port_parser_example
```

## 输出说明

示例程序会解析多种不同格式的IP端口字符串，并输出解析结果。为了避免输出过多，每个测试用例只显示前5个IP端口组合。

如果解析成功，会显示找到的IP端口组合数量以及每个组合的详细信息。如果解析失败，会显示错误信息。