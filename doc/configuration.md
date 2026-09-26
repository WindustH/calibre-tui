# Configuration

calibre-tui reads four TOML files from its configuration directory:

| System | Directory |
| --- | --- |
| Linux | `$XDG_CONFIG_HOME/calibre-tui/`, usually `~/.config/calibre-tui/` |
| macOS | `~/Library/Application Support/calibre-tui/` |
| Windows | `%APPDATA%\calibre-tui\` |

| File | Contents |
| --- | --- |
| `config.toml` | Library path, openers, search translators (this page) |
| `layout.toml` | [Columns and searched fields](layout.md) |
| `keymap.toml` | [Key bindings](keymap.md) |
| `theme.toml` | [Colors](theme.md) |

## How Files Are Created and Updated

- **Missing file**: written with every setting at its default, with explanatory comments.
- **Missing settings** (for example after an upgrade adds new ones): the defaults are filled in and the file is rewritten with comments. Your values are kept, but your own comments and formatting in that file are not.
- **Complete file**: left exactly as you wrote it.
- **Unreadable file** (invalid TOML, an unknown or misspelled setting, a value of the wrong type, or an invalid layout): renamed to `<name>.bak-<timestamp>` and replaced with defaults. A message on stderr names the backup and the error.

A `library_path` that doesn't contain `metadata.db` is reported as an error instead, and `config.toml` is left alone, since the library may just be on an unmounted drive.

## `config.toml`

```toml
library_path = "/home/me/Calibre Library"

[open.commands]
pdf = ["zathura", "{path}"]
epub = ["foliate", "{path}"]
cbz = ["mcomix"]

[filter]
translators = ["pinyin", "romaji"]
pinyin_fuzzy = true
pinyin_fuzzy_groups = [["on", "ong"], ["an", "ang"], ["en", "eng"], ["in", "ing"]]
```

### `library_path`

The Calibre library directory, the one that contains `metadata.db`. On first run it is set to the library found automatically (see [Quick Start](quick-start.md#finding-your-library)). With an empty string, the library is searched for on every start.

### `open.commands`

Programs used to open files, keyed by file extension (case-insensitive; a leading dot is allowed). Formats not listed here, or listed with an empty command, open with the system's default application.

- Each command is an argument list, run directly rather than through a shell.
- `{path}` is replaced with the book's file path, anywhere in any argument. Without `{path}`, the path is added as the last argument.
- The program is started in the background with its output discarded, so it can't disturb the interface and keeps running after calibre-tui exits.

When a book has several formats in Calibre, `Enter` opens the most recently added one.

### `filter`

- `translators`: extra search spellings to index: `pinyin` (default), `romaji`, `german-latin`, `french-latin`, `spanish-latin`, `russian-latin`. Use `[]` for plain text search only. See [Search](search.md#translators).
- `pinyin_fuzzy`: treat the groups below as equivalent in pinyin search (default `true`).
- `pinyin_fuzzy_groups`: groups of interchangeable pinyin fragments; the first item of each group is the canonical form.
