# Architecture

`calibre-tui` is a small synchronous TUI built with ratatui. It combines a Calibre metadata loader, search index, configurable layout, key handling, and themed UI rendering.

## Modules

- `main.rs`: CLI parsing, config loading, terminal setup/restore, stdout path printing.
- `app.rs`: event loop, browser state, selection, command prompt handling, sorting, and opening/printing paths.
- `config.rs`: `config.toml`, Calibre library detection, file-opening options, search translator configuration.
- `config_file.rs`: shared config directory handling, commented TOML writing, missing-field fill-in, incompatible-file backup/reset.
- `layout.rs`: `layout.toml`, visible/searchable columns, validation, and layout compilation.
- `theme.rs`: `theme.toml`, color parsing, and theme sections.
- `keymap.rs`: `keymap.toml` schema and conversion into runtime key bindings.
- `filter.rs`: search index construction, matching, and highlight ranges.
- `sort.rs`: match-field priority and explicit multi-key sort comparison.
- `ui.rs`: ratatui rendering for search box, command prompt, completion list, table, footer, which-key, and F1 help.
- `utils/db.rs`: Calibre SQLite metadata loading.
- `utils/book.rs`: normalized book data used by search and UI.
- `i18n/`: text translators used by the search index.

## Config Strategy

Configs intentionally use serde structs as the source of truth. Files are serialized with `toml::to_string_pretty`, then comments are inserted by field path. That avoids hand-writing TOML values while still generating readable files.

When fields are missing, the file is merged with the current defaults and rewritten. When parsing or validation fails, the old file is backed up and a fresh default is written.
