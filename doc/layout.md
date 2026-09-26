# Layout

`layout.toml` controls the table columns: which fields are shown, in what order and width, and which fields are searched. It is in the [configuration directory](configuration.md).

## Columns

Each `[[columns]]` entry describes one book field:

```toml
[[columns]]
field = "title"
label = "Title"
visible = true
search = true
width = 35
```

| Setting | Meaning | Default |
| --- | --- | --- |
| `field` | `title`, `authors`, `series`, `formats`, or `tags` (required) | |
| `label` | Column header text | the field name |
| `visible` | Show the field as a column | `true` |
| `search` | Match the query against this field | `true` |
| `width` | Relative width; widths are proportions of the total and need not add up to 100 | `1` |

The default layout shows all five fields with widths 35, 20, 18, 12, and 15.

## Order

The order of the entries sets both the column order and the search match priority: when searching, books matching an earlier field are listed before books that only match a later one (see [Search](search.md#result-order)). For example, to rank author matches above title matches, put the `authors` entry first.

## Hidden and Unsearched Fields

- `visible = false`, `search = true`: searched but not shown, useful for tags.
- `visible = true`, `search = false`: shown but ignored by search.
- Both `false`, or leaving the entry out: the field is not used at all.

## Validation

The file is treated as unreadable, backed up as `layout.toml.bak-<timestamp>`, and replaced with the default layout when:

- it has no columns,
- a field appears more than once,
- no column is visible, or
- a visible column has `width = 0`.
