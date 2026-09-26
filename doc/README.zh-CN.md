# Calibre TUI

在终端里搜索 Calibre 书库，然后打开书籍，或把书籍路径输出给 shell 脚本使用。

[English](../README.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Русский](README.ru.md)

https://github.com/user-attachments/assets/7e741b94-80e0-4c61-8479-57e963c01d3e

## 功能

- 边输入边搜索标题、作者、系列、格式和标签。用空格分隔的多个词必须全部匹配。
- 用拉丁字母键盘搜索其他文字：中文用拼音（默认开启），日文假名用罗马字，俄文用拉丁转写，德语、法语、西班牙语可以不输入变音符号。
- 可以多选书籍，然后打开、复制路径，或输出路径并退出，方便在 shell 管道中使用。
- 可以按格式指定打开程序，例如用 `zathura` 打开 PDF；未指定的格式使用系统默认程序。
- 通过 `Ctrl+S` 快捷键或 `sort` 命令按任意字段排序。
- 在带注释的 TOML 文件中自定义列、快捷键（包括带提示的多键序列）和配色。
- Calibre 运行时也能使用；程序只读取书库，从不写入。

## 安装

Arch Linux（AUR）：

```bash
yay -S calibre-tui-bin   # 预编译二进制
yay -S calibre-tui       # 最新正式版，从源码构建
yay -S calibre-tui-git   # 最新 git 版本，从源码构建
```

Homebrew：

```bash
brew install WindustH/tap/calibre-tui          # 预编译二进制
brew install --HEAD WindustH/tap/calibre-tui   # 最新 git 版本
```

每个 GitHub Release 都附有 Linux（x86_64）、macOS（Apple Silicon）和 Windows（x86_64）的预编译二进制。

从源码构建需要 Rust；在 Linux 上还需要 SQLite 开发包（例如 `libsqlite3-dev`）：

```bash
git clone --recursive https://github.com/WindustH/calibre-tui.git
cd calibre-tui
cargo build --release
./target/release/calibre-tui
```

## 使用

运行 `calibre-tui`。程序会自动查找 Calibre 书库；如果找不到，请在 `config.toml` 中设置 `library_path`。

- 直接输入即可搜索，`Backspace` 删除。
- `Up` / `Down` 或鼠标滚轮：移动；`PgUp` / `PgDn`、`Home` / `End`：跳转。
- `Tab`：选中或取消选中当前书籍。`Ctrl+A` 选中全部结果，`Ctrl+X` 清空选择。
- `Enter`：打开选中的书籍；没有选中时打开当前书籍。
- `Ctrl+Y`：把这些书籍的路径复制到剪贴板。
- `Ctrl+P`：输出这些书籍的路径并退出。
- `Ctrl+S` 后按一个字母：排序（`t` 标题、`a` 作者、`s` 系列、`f` 格式、`g` 标签；大写为降序）。
- `Ctrl+T`：命令提示符，例如 `sort authors asc title desc`。
- `F1`：显示所有快捷键。
- `Esc` 或 `Ctrl+C`：退出。

加上 `--exit-on-open` 参数，打开书籍后会自动退出。

路径会逐行输出到 stdout，而界面仍显示在终端上，因此可以直接在脚本中使用：

```bash
zathura "$(calibre-tui)"                    # 打开一本书
calibre-tui | xargs -d '\n' -r cp -t ~/usb  # 复制选中的书籍（GNU xargs）
```

## 配置

首次运行时会创建四个带注释的默认配置文件：

- `config.toml`：书库路径、按格式指定的打开程序、搜索转写
- `layout.toml`：显示哪些列、列的顺序和宽度，以及搜索哪些字段
- `keymap.toml`：快捷键
- `theme.toml`：配色

它们位于 Linux 的 `~/.config/calibre-tui/`、macOS 的 `~/Library/Application Support/calibre-tui/` 和 Windows 的 `%APPDATA%\calibre-tui\`。新版本增加设置项时，会自动用默认值补全。无法读取的文件会另存为 `<文件名>.bak-<时间戳>`，并替换为默认配置。

## 文档

完整文档为英文：

- [快速入门](quick-start.md)
- [操作](controls.md) 与 [命令](commands.md)
- [搜索](search.md)
- [配置](configuration.md)、[布局](layout.md)、[快捷键](keymap.md)、[主题](theme.md)
- [故障排除](troubleshooting.md)
- [架构](architecture.md)（面向贡献者）
