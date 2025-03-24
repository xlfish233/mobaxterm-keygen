# MobaXterm 许可证生成器

这是一个用 Rust 编写的 MobaXterm 许可证生成器。该工具可以生成 MobaXterm 的自定义许可证文件。

## 功能特点

- 支持自定义用户名
- 支持指定 MobaXterm 版本
- 可配置许可证数量
- 生成标准的 `.mxtpro` 许可证文件

## 使用方法

```bash
mobaxterm-keygen [选项]

选项:
    -u, --username <USERNAME>    设置用户名
    -v, --version <VERSION>      设置版本号 (例如: 10.9)
    -c, --count <COUNT>         设置许可证数量 [默认: 1]
    -o, --output <OUTPUT>       设置输出文件名 [默认: "Custom.mxtpro"]
    -h, --help                  显示帮助信息
```

### 示例

生成单个许可证：
```bash
mobaxterm-keygen -u "Your Name" -v 10.9
```

生成多个许可证：
```bash
mobaxterm-keygen -u "Your Name" -v 10.9 -c 5
```

自定义输出文件名：
```bash
mobaxterm-keygen -u "Your Name" -v 10.9 -o "MyLicense.mxtpro"
```

## 构建说明

确保你的系统已安装 Rust 工具链，然后执行：

```bash
cargo build --release
```

编译后的可执行文件将位于 `target/release` 目录下。

## 注意事项

- 生成的许可证文件需要复制到 MobaXterm 的安装目录中
- 请确保指定的版本号与你的 MobaXterm 版本相匹配
- 本工具仅供学习和研究使用

## 许可证

本项目采用 MIT 许可证。 