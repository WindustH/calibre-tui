# Calibre TUI

A Terminal User Interface (TUI) for Calibre, allowing you to search and open books from your Calibre library directly from the terminal.


https://github.com/user-attachments/assets/61323a3a-fc4e-4e2e-92d4-0740a1f7e1f6



### Features

* **Terminal User Interface**: Interact with your Calibre library in a fast and efficient terminal-based UI.
* **Book Search**: Filter through your books by title, author, series, and tags.
* **Pinyin Search**: For users with Chinese book titles, you can search using Pinyin. This feature includes fuzzy search capabilities.
* **Customizable UI**: Configure the look and feel of the application, including colors and table layouts, to your liking.
* **Open Books**: Open books directly from the TUI.

### Install
1.  **On Arch Linux**
    ```bash
    pacman -S calibre-tui
    ```
### Build

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/WindustH/calibre-tui.git
    cd calibre-tui
    ```
2.  **Build the project:**
    ```bash
    cargo build --release
    ```
3.  **Run the application:**
    ```bash
    ./target/release/calibre-tui
    ```

### Configuration

The application can be configured via a `config.toml` file located in your system's configuration directory (`~/.config/calibre-tui/` on Linux).

Here is an example of the default configuration:

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

* `library_path`: The path to your Calibre library. If left empty, the application will attempt to find it in common locations.
* `i18n.filter.pinyin`: Configuration for Pinyin search.
* `ui.filter.table.columns`: Defines the columns in the book list, their appearance, and layout.
* `ui.filter`: Defines the colors for the input box and table borders.

### Usage

* **Start the application:**
    ```bash
    calibre-tui
    ```
* **Keybindings:**
    * `Up`/`Down` arrows or `Scroll`: Navigate the book list.
    * `Enter`: Open the selected book.
    * `Esc` or `Ctrl+C`: Quit the application.
* **Command-line arguments:**
    * `--exit-on-open`: The application will exit after opening a book.
