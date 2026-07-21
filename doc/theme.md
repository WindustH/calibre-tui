# Theme

`theme.toml` controls colors for the main TUI components.

Default location:

- `~/.config/calibre-tui/theme.toml`

The file is generated with comments. The TOML is serialized by the `toml` crate and comments are inserted afterward, so existing values are preserved when missing fields are filled.

## Color Values

Supported color values:

- `reset`
- `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`
- `gray`, `dark_gray`
- `light_red`, `light_green`, `light_yellow`, `light_blue`, `light_magenta`, `light_cyan`
- compact aliases such as `darkgray` and `lightcyan`
- indexed colors such as `ansi:236`
- RGB colors such as `#ffaa00`

Invalid color names fall back to `reset`.

## Sections

Top-level fields:

- `foreground`: base foreground
- `background`: base background
- `accent`: primary emphasis color
- `muted`: secondary text color

Component sections:

- `[search]`: search input box
- `[command]`: command prompt box and inline suggestions
- `[table]`: book list frame, header, and per-field text colors
- `[row]`: hover, selection, and selected-hover row states
- `[highlight]`: search match highlight colors by row state
- `[footer]`: messages and which-key hints
- `[completion]`: command completion list
- `[help]`: F1 help popup

Similar field names have the same meaning across sections. For example, `border` means a component border color, and `title` means a component title color.

## Completion Selection

The selected completion background defaults to `blue` so it visually matches the normal hovered-row behavior:

```toml
[completion]
foreground = "white"
background = "reset"
selected_foreground = "black"
selected_background = "blue"
```

## Which-Key

Which-key hints use fields under `[footer]`:

- `which_key_background`
- `which_key_foreground`
- `which_key_key`
- `which_key_separator`
- `which_key_description`
- `which_key_separator_text`
- `which_key_columns`

The configured column count is reduced automatically on narrow terminals.
