# Configuration

Configuration files are stored in:

- `~/.config/calibre-tui/config.toml`
- `~/.config/calibre-tui/layout.toml`
- `~/.config/calibre-tui/keymap.toml`
- `~/.config/calibre-tui/theme.toml`

All files are generated with comments. The TOML body is produced with serde through the `toml` crate, short scalar arrays are compacted onto one line when they fit, then comments are inserted by field path. This keeps string escaping and table/array formatting handled by the TOML library while still documenting the file.

## Missing Fields

When a file parses successfully but is missing fields introduced by a newer version, `calibre-tui` fills those fields with defaults and rewrites the file with comments.

Existing values are preserved. Missing values are inserted from the current default config. Similar repeated fields, such as `columns.field` or `browser.keymap.on`, are documented once instead of repeating the same comment for every array entry.

## Incompatible Files

If a config file cannot be parsed or no longer matches the expected structure, it is backed up and replaced with a fresh commented default.

Backup names use this form:

```text
config.toml.bak-<timestamp>
layout.toml.bak-<timestamp>
keymap.toml.bak-<timestamp>
theme.toml.bak-<timestamp>
```

Examples of incompatible files:

- invalid TOML syntax
- old keymap format that no longer matches the current keymap structure
- unknown fields rejected by the current config schema
- invalid layout definitions, such as duplicate fields or no visible columns

Runtime errors are still reported normally. For example, an explicit `library_path` that does not contain `metadata.db` is treated as a real library error, not as a reason to replace `config.toml`.

## `config.toml`

Main fields:

- `library_path`: path to the Calibre library directory. Leave empty to auto-detect common locations.
- `open.commands.<format>`: command argv used for a file format. Unconfigured formats use the system opener.
- `filter.translators`: enabled search translators.
- `filter.pinyin_fuzzy`: enable fuzzy pinyin matching.
- `filter.pinyin_fuzzy_groups`: equivalent pinyin fragments. The first item is canonical.

Supported translators:

- `pinyin`
- `romaji`
- `german-latin`
- `french-latin`
- `spanish-latin`
- `russian-latin`

Example:

```toml
library_path = "/home/me/Calibre Library"

[open.commands]
pdf = ["zathura", "{path}"]
epub = ["foliate", "{path}"]

[filter]
translators = ["pinyin", "romaji"]
pinyin_fuzzy = true
pinyin_fuzzy_groups = [
    ["on", "ong"],
    ["an", "ang"],
    ["en", "eng"],
    ["in", "ing"],
]
```

## Other Files

- [Layout](layout.md): `layout.toml`
- [Keymap](keymap.md): `keymap.toml`
- [Theme](theme.md): `theme.toml`
