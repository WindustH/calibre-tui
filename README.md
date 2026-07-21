# Calibre TUI

A terminal UI for searching a Calibre library, selecting books, opening them, or printing their paths for shell workflows.

[中文](doc/README.zh-CN.md) | [日本語](doc/README.ja.md) | [Deutsch](doc/README.de.md) | [Français](doc/README.fr.md) | [Español](doc/README.es.md) | [Русский](doc/README.ru.md)

https://github.com/user-attachments/assets/7e741b94-80e0-4c61-8479-57e963c01d3e

## Features

- Search title, authors, series, formats, and tags with AND semantics across space-separated terms.
- Optional search translators for pinyin, romaji, German/French/Spanish accented Latin folding, and Russian transliteration.
- Configurable `layout.toml` for visible columns, searchable fields, column order, and width ratios.
- Configurable `keymap.toml` with multi-key bindings and which-key hints.
- Command prompt with completions and in-session history.
- Configurable `theme.toml` for search, command, table, row state, highlight, footer, completion, and help colors.
- Configurable format-specific opener commands in `config.toml`.
- Multi-select books, open selected books, or print selected paths and exit with `Ctrl+P`.

## Documentation

Full documentation lives in [doc/index.md](doc/index.md).

- [Quick Start](doc/quick-start.md): install, run, and open books.
- [Controls](doc/controls.md): default key bindings, mouse behavior, help, and which-key.
- [Commands](doc/commands.md): command prompt syntax and sort examples.
- [Configuration](doc/configuration.md): config files, auto-regeneration, and commented defaults.
- [Layout](doc/layout.md): column visibility, search participation, order, and width ratios.
- [Keymap](doc/keymap.md): context-aware keymap format and actions.
- [Theme](doc/theme.md): color syntax and per-component theme fields.
- [Search](doc/search.md): matching, highlighting, translators, and result ordering.
- [Troubleshooting](doc/troubleshooting.md): common library, config, terminal, and key issues.
- [Architecture](doc/architecture.md): module boundaries and dependency notes.

## Installation

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

The Homebrew stable formula downloads a prebuilt release binary. To build the latest git version from source:

```bash
brew install --HEAD WindustH/tap/calibre-tui
```

## Build

```bash
cargo build --release
./target/release/calibre-tui
```

## Quick Usage

Default controls:

- Type to search.
- `Up` / `Down` or mouse wheel: move focus.
- `Tab`: toggle selection for the focused book.
- `Enter`: open selected books, or open the focused book if nothing is selected.
- `Ctrl+P`: print selected/focused book paths to stdout and quit.
- `Ctrl+Y`: copy selected/focused book paths to the system clipboard.
- `Ctrl+S` followed by a field key: apply a common sort.
- `Ctrl+T`: open the command prompt.
- `F1`: show key bindings.
- `Esc` or `Ctrl+C`: quit.

Useful command examples:

```text
sort title asc
sort authors asc title asc
sort formats desc title asc
help
```

Use `--exit-on-open` to quit after opening books:

```bash
calibre-tui --exit-on-open
```

## Configuration Files

On Linux, configuration is stored in:

- `~/.config/calibre-tui/config.toml`
- `~/.config/calibre-tui/layout.toml`
- `~/.config/calibre-tui/keymap.toml`
- `~/.config/calibre-tui/theme.toml`

Files are generated with comments on first run. When newer versions add fields, missing values are filled with defaults and the file is rewritten with comments. Incompatible files are backed up as `*.bak-<timestamp>` and replaced with fresh commented defaults.
