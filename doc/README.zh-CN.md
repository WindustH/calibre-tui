# Calibre TUI

一个用于搜索 Calibre 书库并从终端打开书籍的轻量 TUI。

[English](../README.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Русский](README.ru.md)

### 功能

* 按标题、作者、系列和标签搜索。空格分隔的多个词会按逻辑与匹配。
* 支持拼音、日语、德语、法语、西班牙语、俄语 translator。
* 可以同时启用多个 translator。原文搜索始终启用。
* 支持多选书籍、打开书籍，或只把书籍路径输出到 stdout。
* 可以在 `config.toml` 中按格式配置打开命令。
* 界面保持紧凑固定，不提供复杂 UI 和主题配置。

### 构建

```bash
cargo build --release
./target/release/calibre-tui
```

### 配置

配置文件位于 Linux 的 `~/.config/calibre-tui/`。如果不存在，程序会写入默认文件。

```toml
library_path = ""

[open.commands]
pdf = ["zathura", "{path}"]
epub = ["foliate", "{path}"]

[filter]
translators = ["pinyin", "romaji", "german-latin", "french-latin", "spanish-latin", "russian-latin"]
pinyin_fuzzy = true
pinyin_fuzzy_groups = [
    ["on", "ong"],
    ["an", "ang"],
    ["en", "eng"],
    ["in", "ing"]
]
```

* `library_path`：Calibre 书库路径。留空时会自动查找常见位置。
* `open.commands.<format>`：某种格式使用的打开命令 argv。未配置的格式使用系统默认打开方式；如果没有写 `{path}`，路径会自动追加到最后。
* `filter.translators`：启用的搜索 translator。支持 `pinyin`、`romaji`、`german-latin`、`french-latin`、`spanish-latin`、`russian-latin`。
* `filter.pinyin_fuzzy`：是否启用拼音模糊匹配。
* `filter.pinyin_fuzzy_groups`：等价拼音片段，每组第一个值是归一化后的标准形式。

Translator 行为：

* `pinyin`：中文汉字可用拼音搜索，并支持可配置的模糊拼音。
* `romaji`：日语假名可用 romaji 搜索，全角 ASCII 会归一化。没有词典时不会推断任意汉字读音，但原文搜索仍然可用。
* `german-latin`：德语 `ä/ö/ü/ß` 可用 `ae/oe/ue/ss` 匹配。
* `french-latin`：折叠法语重音字母，例如 `étranger` 可用 `etranger` 匹配。
* `spanish-latin`：折叠西班牙语重音字母，例如 `niñez` 可用 `ninez` 匹配。
* `russian-latin`：俄语西里尔字母可用拉丁转写搜索，例如 `Преступление` 可用 `prestuplenie` 匹配。

### 快捷键

`keymap.toml` 控制快捷键：

```toml
[browser]
keymap = [
  { on = "esc", run = "quit", desc = "Quit" },
  { on = "enter", run = "open", desc = "Open selected books" },
]

[global]
keymap = [
  { on = "f1", run = "help", desc = "Show key bindings" },
  { on = "ctrl-t", run = "command", desc = "Enter command" },
]
```

当前按键命名、按键序列和动作说明见 [Keymap](keymap.md)。

### 使用

* `Up` / `Down` 或鼠标滚轮：移动光标。
* `PgUp` / `PgDown`：翻页移动。
* `Home` / `End`：跳到第一项或最后一项。
* `Tab`：切换当前书籍的多选状态，选中后自动下移一格。
* `Ctrl+A`：选中当前过滤结果里的所有书籍。
* `Ctrl+X`：清空选中。
* `Ctrl+P`：把选中书籍路径输出到 stdout 并退出。
* `Ctrl+Y`：把选中书籍路径复制到系统剪切板；没有选中时复制光标所在书籍。
* `Ctrl+S` 后接排序键：应用常用排序。
* `Ctrl+T`：进入命令模式。
* `F1`：显示按键帮助。
* `Enter`：打开选中的书籍；如果没有选中书籍，则打开光标所在书籍。
* `Esc` 或 `Ctrl+C`：退出。
* `--exit-on-open`：打开书籍后退出。

最新完整文档见 [doc/index.md](index.md)。
