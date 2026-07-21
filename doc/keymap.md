# Keymap

`keymap.toml` controls keyboard shortcuts, including context-aware bindings, multi-key sequences, which-key hints, prompt editing, completion selection, and command history actions.

Default location:

- `~/.config/calibre-tui/keymap.toml`

## Contexts

The file is split into sections:

- `browser`: active while browsing and searching books.
- `detail`: reserved for future detail views.
- `input`: active while the command prompt is open.
- `global`: active from normal browsing contexts.

Serde writes the default file with array-of-table entries:

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

The compact form is also valid TOML:

```toml
[browser]
keymap = [
  { on = "enter", run = "open", desc = "Open selected books" },
  { on = ["ctrl-s", "t"], run = "sort title asc", desc = "Sort title ascending" },
]
```

## Key Names

Supported key names include:

- characters such as `a`, `t`, `T`
- `enter`, `esc`, `tab`, `backtab`, `backspace`, `delete`, `insert`, `space`
- `left`, `right`, `up`, `down`, `home`, `end`, `pgup`, `pgdn`
- function keys such as `f1`
- modifiers such as `ctrl-c`, `ctrl-t`, `alt-x`
- Yazi-style angle names, such as `<Enter>` or `<C-c>`

`on` can be a single key or a key sequence. For a sequence, which-key hints are shown after the prefix key.

## Browser Actions

Common browser actions:

- `quit`
- `open`
- `print_paths`
- `copy_paths`
- `move_up`, `move_down`
- `page_up`, `page_down`
- `jump_start`, `jump_end`
- `toggle_selection`
- `select_all`
- `clear_selection`
- `delete_input`
- `command`
- `help`
- `sort <field> [asc|desc] [field] [asc|desc] ...`

Sort actions use the same syntax as the command prompt, without the leading colon.

## Input Actions

Prompt actions:

- `cancel`, `submit`
- `backspace`, `delete`
- `move_left`, `move_right`, `move_start`, `move_end`
- `kill_before_cursor`, `kill_after_cursor`
- `completion_next`, `completion_previous`
- `history_previous`, `history_next`

## Defaults

Important defaults:

- `Esc`, `Ctrl+C`: quit
- `Enter`: open selected/focused books
- `Ctrl+P`: print paths and quit
- `Ctrl+Y`: copy paths to the system clipboard
- `Ctrl+T`: command prompt
- `F1`: key binding help
- `Ctrl+S` followed by a field key: common sorts

When a key sequence is waiting for its next key, `Esc` clears that waiting state first instead of quitting.
