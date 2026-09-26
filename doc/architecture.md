# Architecture

calibre-tui is a synchronous ratatui application. At startup it loads the Calibre metadata into memory and builds a search index; after that, every keystroke searches the index and redraws.

## Modules

| Module | Responsibility |
| --- | --- |
| `main.rs` | CLI arguments, loading the four config files, running the app, printing paths |
| `terminal.rs` | Raw mode, alternate screen, and mouse capture; restoring the terminal on exit, error, and panic; drawing on stderr when stdout is redirected |
| `app/mod.rs` | `App`: owns all state, runs the event loop, routes events by mode (book list, command prompt, key help) and dispatches keymap actions |
| `app/browser.rs` | Book list state: search input, current results, focus, scroll position, and selected books |
| `app/command.rs` | Parsing and completing prompt commands (`sort`, `help`) |
| `app/launch.rs` | Opening books with configured commands or the system opener; clipboard |
| `library/mod.rs` | `Book`, `BookField` and the display text of each field; finding a Calibre library |
| `library/db.rs` | Loading books from `metadata.db` (read-only, with an immutable fallback while Calibre holds a lock) |
| `search/mod.rs` | `BookSearch`: per-field index, term matching, highlight ranges, and match rank |
| `search/translit/` | Plain-text normalization and the pinyin, romaji, German, Latin, and Russian translators |
| `sort.rs` | `SortSpec` parsing and `BookOrder`, the library sorted once per sort spec |
| `ui/mod.rs` | Screen layout, search and command boxes, completion list, footer, help popup |
| `ui/table.rs` | The book table: visible rows only, row styles, match highlighting |
| `config.rs` | `config.toml` schema |
| `config_file.rs` | Shared handling of the TOML files: commented defaults, filling missing settings, backups |
| `layout.rs`, `keymap.rs`, `theme.rs` | Schemas for `layout.toml`, `keymap.toml`, and `theme.toml` and their conversion to runtime types |

Key handling, prompt editing, command history and completion state, which-key hints, and the popup widgets come from the `framework-tui` crate in `crates/framework-tui` (a git submodule).

## Data Flow

1. `library::load_books` reads every book once. `Book::field_text` defines each field's display text; the search index, the table, and sorting all use it, which keeps highlight positions aligned with what is drawn.
2. `BookSearch::new` indexes each searchable field as plain lowercase text plus one version per enabled translator (versions identical to the plain text are not stored). Each version records where every source character's output ends, so a match found in `zhongguo` maps back to the characters `中国`.
3. On input, `BookSearch::search` matches every term against every searchable field. When the new query can only narrow the previous one (for example one more typed character), only the previous matches are rescanned.
4. `BookOrder::sort` orders results by match rank and then by each book's precomputed position under the current sort spec, so sorting compares integers.
5. `ui::draw` builds rows only for the visible part of the table and reports the scroll position back to `Browser`.

The event loop blocks until input arrives, handles all queued events, and redraws once if anything changed; typed characters mark the search stale and it reruns once per batch.

## Configuration Files

Each config struct is its own serde schema with defaults, and implements `CommentedToml` to attach comments by field path. `config_file::load_toml_or_reset_with` writes missing files, fills in missing settings (rewriting the file only then), and backs up files that fail to parse or validate. `layout.toml` is validated and compiled into a `Layout`; `keymap.toml` becomes `framework_tui::KeyBindings`.
