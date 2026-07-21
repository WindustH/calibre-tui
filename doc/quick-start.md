# Quick Start

`calibre-tui` opens directly into the searchable book browser. It reads Calibre's `metadata.db`, builds an in-memory search index, and opens selected book files with the system opener unless a format-specific opener command is configured.

## Install

Arch Linux:

```bash
yay -S calibre-tui-bin
```

Homebrew:

```bash
brew install WindustH/tap/calibre-tui
```

Build from source:

```bash
cargo build --release
./target/release/calibre-tui
```

## First Run

On first run, the app creates these files:

- `~/.config/calibre-tui/config.toml`
- `~/.config/calibre-tui/layout.toml`
- `~/.config/calibre-tui/keymap.toml`
- `~/.config/calibre-tui/theme.toml`

Generated files include comments. If a later version adds new fields, missing values are written back with defaults and comments.

To use specific commands for some formats, add entries under `open.commands` in `config.toml`:

```toml
[open.commands]
pdf = ["zathura", "{path}"]
epub = ["foliate", "{path}"]
```

Leave the table empty to use the system opener for every format.

## Library Detection

`library_path` in `config.toml` points to your Calibre library directory. If it is empty, `calibre-tui` tries common locations:

- `~/Calibre Library`
- `~/Calibre-Bibliothek`
- the `library_path` from `~/.config/calibre/global.py.json`
- `~/Documents/Calibre Library`

The directory must contain `metadata.db`.

## Basic Workflow

1. Type search terms.
2. Move with `Up` / `Down` or mouse wheel.
3. Press `Tab` to select multiple books.
4. Press `Enter` to open selected books, or the focused book if nothing is selected.
5. Press `Ctrl+P` to print selected/focused paths to stdout and quit.

Use `--exit-on-open` when you want `Enter` to open books and quit immediately:

```bash
calibre-tui --exit-on-open
```

## Command Prompt

Press `Ctrl+T` to open the command prompt. `Tab` and `Shift+Tab` select completion candidates, `Enter` first applies an active completion and then runs the command, and `Up` / `Down` browse in-session command history.

Example:

```text
sort authors asc title asc
```
