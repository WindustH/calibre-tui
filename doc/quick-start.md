# Quick Start

`calibre-tui` opens straight into a searchable list of the books in your Calibre library. It reads Calibre's `metadata.db` (never writing to it), so it works while Calibre is running.

## Install

```bash
yay -S calibre-tui-bin                   # Arch Linux (AUR)
brew install WindustH/tap/calibre-tui    # Homebrew
```

Prebuilt binaries for Linux, macOS, and Windows are attached to each GitHub release. To build from source (on Linux you also need the SQLite development package, such as `libsqlite3-dev`):

```bash
git clone --recursive https://github.com/WindustH/calibre-tui.git
cd calibre-tui
cargo build --release
```

## Finding Your Library

On first run, calibre-tui looks for a library in this order and saves the first match as `library_path` in `config.toml`:

1. The library Calibre itself last opened (`library_path` in Calibre's `global.py.json`, which lives in `~/.config/calibre/` on Linux, `~/Library/Preferences/calibre/` on macOS, `%APPDATA%\calibre\` on Windows, or `$CALIBRE_CONFIG_DIRECTORY`).
2. `~/Calibre Library`
3. `~/Calibre-Bibliothek`
4. `Calibre Library` in your Documents folder

A library is a directory containing `metadata.db`. To use a different one, edit `library_path`; if you set it to an empty string, the search above runs on every start. See [Configuration](configuration.md) for where `config.toml` is.

## Basic Workflow

1. Type to filter the list. Every space-separated word must match.
2. Move with `Up` / `Down` or the mouse wheel.
3. Press `Tab` to select books (the search box title shows how many are selected).
4. Press `Enter` to open the selected books, or the focused book if none are selected.

Instead of opening, `Ctrl+Y` copies the paths to the clipboard and `Ctrl+P` prints them to stdout and quits:

```bash
zathura "$(calibre-tui)"
```

Run `calibre-tui --exit-on-open` to quit as soon as books are opened.

## Choosing Openers

By default each file opens with the system's default application. To pick a program per format, add it to `config.toml`:

```toml
[open.commands]
pdf = ["zathura", "{path}"]
epub = ["foliate", "{path}"]
```

## Next Steps

- `F1` shows every key binding; [Controls](controls.md) lists the defaults.
- `Ctrl+T` opens the command prompt, for example `sort authors asc title asc`. See [Commands](commands.md).
- Chinese pinyin search is on by default; see [Search](search.md) to enable romaji, Russian, or accent-insensitive search.
