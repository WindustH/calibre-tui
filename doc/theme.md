# Theme

`theme.toml` sets the interface colors. It is in the [configuration directory](configuration.md) and is created with the defaults and comments.

## Color Values

- `reset`: the terminal's default color
- Named colors: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `gray`, `dark_gray`, and `light_red`, `light_green`, `light_yellow`, `light_blue`, `light_magenta`, `light_cyan`. Spellings like `darkgray`, `dark_grey`, and `lightcyan` also work.
- `ansi:<0-255>`: a 256-color palette index, for example `ansi:236`
- `#rrggbb`: an RGB color, for example `#ffaa00`

Values are case-insensitive. An unrecognized value falls back to `reset`.

## Fields

Fields with the same name mean the same thing in every section: `border` is a box border, `title` a box title, `text` typed text.

| Section | Fields | Colors for |
| --- | --- | --- |
| top level | `foreground`, `background` | Default text and the screen background |
| top level | `accent`, `muted` | Reserved; currently unused |
| `[search]` | `border`, `title`, `text` | The search box |
| `[command]` | `border`, `title`, `text`, `prefix`, `suggestion` | The command prompt; `prefix` is the `:` and `suggestion` the inline completion |
| `[table]` | `border`, `title`, `header` | The book list frame and column headers |
| `[table]` | `title_field`, `authors_field`, `series_field`, `formats_field`, `tags_field` | Text of each column in unselected, unfocused rows |
| `[row]` | `hover_foreground`, `hover_background` | The focused row |
| `[row]` | `selected_foreground`, `selected_background` | Selected rows |
| `[row]` | `selected_hover_foreground`, `selected_hover_background` | A row that is both selected and focused |
| `[highlight]` | `normal`, `hover`, `selected`, `selected_hover` | Search matches (bold) in each of those row states |
| `[footer]` | `message` | Status messages |
| `[footer]` | `which_key_background`, `which_key_foreground`, `which_key_key`, `which_key_separator`, `which_key_description` | Which-key hints for pending key sequences |
| `[footer]` | `which_key_separator_text`, `which_key_columns` | Text between a key and its description, and the preferred number of hint columns (fewer on narrow terminals) |
| `[completion]` | `foreground`, `background`, `selected_foreground`, `selected_background` | The command completion list |
| `[help]` | `background`, `border`, `key`, `description`, `muted` | The `F1` key binding popup; `muted` is its closing hint |

## Example

A darker focused row with an orange match highlight:

```toml
[row]
hover_foreground = "white"
hover_background = "ansi:238"

[highlight]
hover = "#ffaa00"
```

Leave out any field to keep its default; missing fields are added back to the file with their defaults on the next start.
