# Controls

These are the default bindings; all of them can be changed in [`keymap.toml`](keymap.md). Press `F1` at any time to see the bindings currently in effect.

## Book List

| Key | Action |
| --- | --- |
| Any character | Add to the search |
| `Backspace` | Delete the last search character |
| `Up` / `Down`, mouse wheel | Move focus (wraps around at either end) |
| `PgUp` / `PgDn` | Move one page |
| `Home` / `End` | Jump to the first / last result |
| `Tab` | Select or unselect the focused book, then move down |
| `Ctrl+A` | Select every book in the current results |
| `Ctrl+X` | Clear the selection |
| `Enter` | Open the selected books, or the focused book if none are selected |
| `Ctrl+Y` | Copy the paths of those books to the clipboard |
| `Ctrl+P` | Print the paths of those books to stdout and quit |
| `Ctrl+S`, then a letter | Sort (see below) |
| `Ctrl+T` | Open the command prompt |
| `F1` | Show key bindings |
| `Esc`, `Ctrl+C` | Quit |

Selections are kept while you change the search, so you can collect books from several searches; the count is shown in the search box title. `Enter` clears the selection after opening.

Books that have no file in Calibre (metadata-only entries) can't be opened; a message is shown instead, and `Ctrl+P` / `Ctrl+Y` skip them.

## Sorting With `Ctrl+S`

After `Ctrl+S`, the footer lists the available follow-up keys:

| Key | Sort by | Key | Sort by |
| --- | --- | --- | --- |
| `t` | title, ascending | `T` | title, descending |
| `a` | authors, ascending | `A` | authors, descending |
| `s` | series, ascending | `S` | series, descending |
| `f` | formats, ascending | `F` | formats, descending |
| `g` | tags, ascending | `G` | tags, descending |

`Esc` cancels a pending key sequence without quitting. For sorts on several fields, use the [`sort` command](commands.md).

## Command Prompt

| Key | Action |
| --- | --- |
| `Enter` | Accept the highlighted completion, or run the command |
| `Esc` | Close the prompt |
| `Tab` / `Shift+Tab` | Next / previous completion |
| `Up` / `Down` | Previous / next command from this session |
| `Left` / `Right` | Move the cursor |
| `Home` / `Ctrl+A`, `End` / `Ctrl+E` | Move to the start / end |
| `Backspace` / `Delete` | Delete before / under the cursor |
| `Ctrl+U` / `Ctrl+K` | Delete everything before / after the cursor |
| `F1` | Show the prompt's key bindings |

## Key Binding Help

`F1` (or the `help` command) lists the bindings for the current context. Close it with `Esc`, `q`, `Enter`, or `F1`.
