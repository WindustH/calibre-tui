# Calibre TUI

一个用于 Calibre 的终端用户界面（TUI），让您可以直接从终端搜索和打开您的 Calibre 书库中的书籍。

### 功能

* **终端用户界面**：通过一个快速高效的基于终端的用户界面与您的 Calibre 书库进行交互。
* **书籍搜索**：按书名、作者、系列和标签筛选您的书籍。
* **拼音搜索**：对于拥有中文书名的用户，您可以使用拼音进行搜索。此功能包括模糊搜索功能。
* **可定制的用户界面**：根据您的喜好配置应用程序的外观，包括颜色和表格布局。
* **打开书籍**：直接从 TUI 中打开书籍。

### 安装

1.  **克隆代码库：**
    ```bash
    git clone [https://github.com/windusth/calibre-tui.git](https://github.com/windusth/calibre-tui.git)
    cd calibre-tui
    ```
2.  **构建项目：**
    ```bash
    cargo build --release
    ```
3.  **运行应用程序：**
    ```bash
    ./target/release/calibre-tui
    ```

### 配置

该应用程序可以通过位于您系统配置目录中（在 Linux 上为 `~/.config/calibre-tui/`）的 `config.toml` 文件进行配置。

以下是默认配置的示例：

```toml
[app]
library_path = ""
# ---------------------
[i18n.filter.pinyin]
enabled = true
fuzzy_enabled = true
fuzzy_groups = [
    ["ong", "on"],
    ["an", "ang"],
    ["en", "eng"],
    ["in", "ing"]
]
# ---------------------
[[ui.filter.table.columns]]
label = "title"
position = 0
ratio = 40
fg = "White"
hovered_fg = "White"
hovered_bg = "Blue"
label_fg = "Blue"
highlighted_fg = "Red"
hovered_highlighted_fg="Yellow"
# ---------------------
[[ui.filter.table.columns]]
label = "authors"
position = 1
ratio = 20
fg = "Cyan"
hovered_fg = "White"
hovered_bg = "Blue"
label_fg = "Blue"
highlighted_fg= "Red"
hovered_highlighted_fg="Yellow"
# ---------------------
[[ui.filter.table.columns]]
label = "series"
position = 2
ratio = 20
fg = "White"
hovered_fg = "White"
hovered_bg = "Blue"
label_fg = "Blue"
highlighted_fg= "Red"
hovered_highlighted_fg="Yellow"
# ---------------------
[[ui.filter.table.columns]]
label = "tags"
position = 3
ratio = 20
fg = "Cyan"
hovered_fg = "White"
hovered_bg = "Blue"
label_fg = "Blue"
highlighted_fg= "Red"
hovered_highlighted_fg="Yellow"
# ---------------------

[ui.filter]
inputbox.border.fg = "Blue"
table.border.fg = "Blue"
inputbox.title.fg = "Blue"
inputbox.fg = "White"
table.title.fg = "Blue"
```

* `library_path`：您的 Calibre 书库的路径。如果留空，应用程序将尝试在常用位置查找。
* `i18n.filter.pinyin`：拼音搜索的配置。
* `ui.filter.table.columns`：定义书籍列表中的列、它们的外观和布局。
* `ui.filter`：定义输入框和表格边框的颜色。

### 使用

* **启动应用程序：**
    ```bash
    calibre-tui
    ```
* **按键绑定：**
    * `上`/`下` 箭头或 `滚动`：浏览书籍列表。
    * `回车`：打开选定的书籍。
    * `Esc` 或 `Ctrl+C`：退出应用程序。
* **命令行参数：**
    * `--exit-on-open`：打开一本书后，应用程序将退出。
