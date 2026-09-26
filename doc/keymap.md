# Keymap

`keymap.toml` holds every key binding, in the [configuration directory](configuration.md). The defaults are listed in [Controls](controls.md), and `F1` shows the bindings in effect.

## Format

Bindings are grouped by context, and each binding has three fields:

```toml
[[browser.keymap]]
on = "enter"
run = "open"
desc = "Open selected books"

[[browser.keymap]]
on = ["ctrl-s", "t"]
run = "sort title asc"
desc = "Sort title ascending"
```

- `on`: one key, or a list of keys pressed one after another. While a sequence is incomplete, the footer lists the possible next keys with their `desc` (which-key); `Esc` cancels it.
- `run`: the action to run.
- `desc`: text shown in the `F1` help and in which-key hints.

The same bindings can be written with inline tables:

```toml
[browser]
keymap = [
  { on = "enter", run = "open", desc = "Open selected books" },
  { on = ["ctrl-s", "t"], run = "sort title asc", desc = "Sort title ascending" },
]
```

## Contexts

| Section | Active |
| --- | --- |
| `browser` | In the book list |
| `global` | In the book list too; if a key is bound in both sections, the `global` binding wins |
| `input` | While the command prompt is open (`browser` and `global` are not used there) |
| `detail` | Reserved; currently unused |

A printable key bound in `browser` or `global` (for example `q`) can no longer be typed into the search.

## Key Names

- Characters: `a`, `T` (uppercase means with Shift), `/`, and so on; `space` for the space bar.
- Special keys: `enter`, `esc`, `tab`, `backtab` (Shift+Tab), `backspace`, `delete`, `insert`, `left`, `right`, `up`, `down`, `home`, `end`, `pgup`, `pgdn` (`pageup` and `pagedown` also work), `f1` to `f12`.
- Modifiers: `ctrl-x`, `alt-x`, for example `ctrl-t`, `alt-backspace`.
- Vim-style names in angle brackets also work: `<Enter>`, `<Esc>`, `<Space>`, `<S-Tab>`, `<PageDown>`, `<C-c>`, `<A-x>`, `<F1>`.

Some terminals can't distinguish every combination; `Ctrl+/` and `Ctrl+;`, for instance, are reported inconsistently.

## Book List Actions

| Action | Effect |
| --- | --- |
| `quit` | Quit |
| `open` | Open the selected books, or the focused book |
| `print_paths` | Print the paths of the selected or focused books to stdout and quit |
| `copy_paths` | Copy those paths to the clipboard |
| `move_up`, `move_down` | Move focus by one row |
| `page_up`, `page_down` | Move focus by one page |
| `jump_start`, `jump_end` | Focus the first or last result |
| `toggle_selection` | Select or unselect the focused book and move down |
| `select_all` | Select every book in the current results |
| `clear_selection` | Clear the selection |
| `delete_input` | Delete the last search character |
| `command` | Open the command prompt |
| `help` | Show key bindings |
| `sort <field> [asc\|desc] ...` | Sort, with the same syntax as the [`sort` command](commands.md#sort) |

## Command Prompt Actions

| Action | Effect |
| --- | --- |
| `submit` | Accept the highlighted completion, or run the command |
| `cancel` | Close the prompt |
| `backspace`, `delete` | Delete before / under the cursor |
| `move_left`, `move_right`, `move_start`, `move_end` | Move the cursor |
| `kill_before_cursor`, `kill_after_cursor` | Delete everything before / after the cursor |
| `completion_next`, `completion_previous` | Choose a completion |
| `history_previous`, `history_next` | Recall commands from this session |
| `help` | Show key bindings |
