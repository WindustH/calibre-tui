# Layout

`layout.toml` controls which metadata fields appear as columns, which fields participate in search, the column order, and relative column widths.

Default location:

- `~/.config/calibre-tui/layout.toml`

## Column Entries

Each `[[columns]]` entry describes one book metadata field:

```toml
[[columns]]
field = "title"
label = "title"
visible = true
search = true
width = 35
```

Fields:

- `field`: one of `title`, `authors`, `series`, `formats`, or `tags`.
- `label`: table header text.
- `visible`: show this field as a table column.
- `search`: include this field in search matching and highlighting.
- `width`: relative table width. Values are proportions and do not need to add up to 100.

## Order

The order of `[[columns]]` entries controls two things:

- visible table column order
- search match priority

If `title` appears before `formats`, a book that matches in `title` is ranked before a book that only matches in `formats`, before normal sort keys are applied.

## Search Without Display

Set `visible = false` and `search = true` to search a field without showing it as a table column:

```toml
[[columns]]
field = "tags"
label = "tags"
visible = false
search = true
width = 15
```

Set both `visible = false` and `search = false` to fully disable a field.

## Validation

The layout is considered incompatible and is regenerated from defaults when:

- no columns are defined
- a field appears more than once
- no column is visible
- a visible column has `width = 0`

The old file is backed up as `layout.toml.bak-<timestamp>` before a default file is written.
