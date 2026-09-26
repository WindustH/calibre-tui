# Calibre TUI

A terminal UI for searching your Calibre library, then opening books or printing their paths for shell scripts.

[中文](doc/README.zh-CN.md) | [日本語](doc/README.ja.md) | [Deutsch](doc/README.de.md) | [Français](doc/README.fr.md) | [Español](doc/README.es.md) | [Русский](doc/README.ru.md)

https://github.com/user-attachments/assets/7e741b94-80e0-4c61-8479-57e963c01d3e

## Features

- Instant search as you type across title, authors, series, formats, and tags. Space-separated terms must all match.
- Search other scripts from a Latin keyboard: Chinese by pinyin (on by default), Japanese kana by romaji, Russian by transliteration, and German, French, or Spanish without accents.
- Select several books, open them, copy their paths, or print the paths and exit for use in shell pipelines.
- Choose an opener per format, such as `zathura` for PDF, or fall back to the system default.
- Sort by any field with `Ctrl+S` shortcuts or the `sort` command.
- Customize columns, key bindings (including multi-key sequences with hints), and colors in commented TOML files.
- Works while Calibre is running; the library is only ever read.

## Installation

Arch Linux (AUR):

```bash
yay -S calibre-tui-bin   # prebuilt binary
yay -S calibre-tui       # latest release, built from source
yay -S calibre-tui-git   # latest git version, built from source
```

Homebrew:

```bash
brew install WindustH/tap/calibre-tui          # prebuilt binary
brew install --HEAD WindustH/tap/calibre-tui   # latest git version
```

Prebuilt binaries for Linux (x86_64), macOS (Apple Silicon), and Windows (x86_64) are attached to each GitHub release.

To build from source you need Rust and, on Linux, the SQLite development package (for example `libsqlite3-dev`):

```bash
git clone --recursive https://github.com/WindustH/calibre-tui.git
cd calibre-tui
cargo build --release
./target/release/calibre-tui
```

## Usage

Run `calibre-tui`. It finds your Calibre library automatically; if it can't, set `library_path` in `config.toml`.

- Type to search; `Backspace` deletes.
- `Up` / `Down` or the mouse wheel: move; `PgUp` / `PgDn`, `Home` / `End`: jump.
- `Tab`: select or unselect the focused book. `Ctrl+A` selects all results, `Ctrl+X` clears.
- `Enter`: open the selected books, or the focused one if none are selected.
- `Ctrl+Y`: copy their paths to the clipboard.
- `Ctrl+P`: print their paths and quit.
- `Ctrl+S`, then a letter: sort (`t` title, `a` authors, `s` series, `f` formats, `g` tags; uppercase for descending).
- `Ctrl+T`: command prompt, for example `sort authors asc title desc`.
- `F1`: show all key bindings.
- `Esc` or `Ctrl+C`: quit.

Pass `--exit-on-open` to quit after opening books.

Printed paths go to stdout, one per line, while the interface stays on the terminal, so you can use them in scripts:

```bash
zathura "$(calibre-tui)"                    # open one book
calibre-tui | xargs -d '\n' -r cp -t ~/usb  # copy the selected books (GNU xargs)
```

## Configuration

Four files are created with commented defaults on first run:

- `config.toml`: library path, per-format openers, search translators
- `layout.toml`: columns, their order and widths, and which fields are searched
- `keymap.toml`: key bindings
- `theme.toml`: colors

They live in `~/.config/calibre-tui/` on Linux, `~/Library/Application Support/calibre-tui/` on macOS, and `%APPDATA%\calibre-tui\` on Windows. When an update adds settings, they are filled in with defaults. A file that can no longer be read is saved as `<name>.bak-<timestamp>` and replaced with defaults.

## Documentation

- [Quick Start](doc/quick-start.md)
- [Controls](doc/controls.md) and [Commands](doc/commands.md)
- [Search](doc/search.md)
- [Configuration](doc/configuration.md), [Layout](doc/layout.md), [Keymap](doc/keymap.md), [Theme](doc/theme.md)
- [Troubleshooting](doc/troubleshooting.md)
- [Architecture](doc/architecture.md), for contributors
