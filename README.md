# Calibre TUI

A small terminal UI for searching a Calibre library and opening books from the terminal.

[中文](doc/README.zh-CN.md) | [日本語](doc/README.ja.md) | [Deutsch](doc/README.de.md) | [Français](doc/README.fr.md) | [Español](doc/README.es.md) | [Русский](doc/README.ru.md)

https://github.com/user-attachments/assets/a7d2157c-68bc-468f-a478-57645b414864

### Features

* Search by title, author, series, and tags. Space-separated terms are matched with logical AND.
* Optional translators for Pinyin, Japanese, German, French, Spanish, and Russian search.
* Multiple translators can be enabled at the same time. Original text search is always enabled.
* Fixed, compact TUI layout with no UI/theme configuration.
* Open the selected book directly from the TUI.

### Installation

Arch Linux AUR:

```bash
yay -S calibre-tui-bin
```

Alternative AUR packages:

```bash
yay -S calibre-tui      # build the latest stable release from source
yay -S calibre-tui-git  # build the latest git version from source
```

Homebrew:

```bash
brew install WindustH/tap/calibre-tui
```

The Homebrew stable formula downloads a prebuilt release binary. To build the
latest git version from source:

```bash
brew install --HEAD WindustH/tap/calibre-tui
```

### Build

```bash
cargo build --release
./target/release/calibre-tui
```

### Configuration

The config files are stored in `~/.config/calibre-tui/` on Linux. If they do not exist, the app writes default files.

```toml
library_path = ""

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

* `library_path`: Path to your Calibre library. Leave it empty to auto-detect common Calibre locations.
* `filter.translators`: Search translators to enable. Supported values are `pinyin`, `romaji`, `german-latin`, `french-latin`, `spanish-latin`, and `russian-latin`.
* `filter.pinyin_fuzzy`: Enables fuzzy Pinyin matching.
* `filter.pinyin_fuzzy_groups`: Equivalent Pinyin fragments. The first item in each group is the canonical form.

Translator behavior:

* `pinyin`: Chinese Hanzi can be searched by Pinyin, with optional fuzzy groups.
* `romaji`: Japanese kana can be searched by romaji. Full-width ASCII is normalized. Arbitrary Kanji readings are not inferred without a dictionary, but original text search still works.
* `german-latin`: Accented Latin characters are folded, with German `ä/ö/ü/ß` matched as `ae/oe/ue/ss`.
* `french-latin`: Accented Latin characters are folded, so `étranger` can be found by `etranger`.
* `spanish-latin`: Accented Latin characters are folded, so `niñez` can be found by `ninez`.
* `russian-latin`: Cyrillic can be searched by Latin transliteration, for example `Преступление` by `prestuplenie`.

`keymap.toml` controls keyboard shortcuts:

```toml
quit = ["esc", "ctrl-c"]
submit = ["enter"]
move_up = ["up"]
move_down = ["down"]
page_up = ["pgup"]
page_down = ["pgdown"]
jump_start = ["home"]
jump_end = ["end"]
toggle_selection = ["tab"]
select_all = ["ctrl-a"]
clear_selection = ["ctrl-x"]
delete_input = ["backspace"]
```

Key names support common keys like `enter`, `esc`, `tab`, `backspace`, `up`, `down`, `left`, `right`, `home`, `end`, `page-up`, `page-down`, `delete`, `insert`, `space`, single characters, and modifiers such as `ctrl-a`, `alt-x`, or `shift-tab`.
Bindings can also be key sequences separated by spaces. For example, `jump_start = ["home", "ctrl-g g"]` and `jump_end = ["end", "ctrl-g G"]`.

### Usage

* `Up` / `Down` or mouse scroll: Move the cursor.
* `PgUp` / `PgDown`: Move by one page.
* `Home` / `End`: Jump to the first or last filtered result.
* `Tab`: Toggle multi-selection for the current book.
* `Ctrl+A`: Select all current filtered results.
* `Ctrl+X`: Clear all selected books.
* `Enter`: Open selected books, or the cursor book if nothing is selected.
* `Esc` or `Ctrl+C`: Quit.
* `--exit-on-submit`: Quit after submitting books.
* `--print-path`: Print selected book paths to stdout instead of opening them.
