# Commands

Open the command prompt with `Ctrl+T`.

While the prompt is open:

- `Tab`: select the next completion candidate
- `Shift+Tab`: select the previous completion candidate
- `Enter`: apply the selected completion; press `Enter` again to run when the command is complete
- `Up` / `Down`: browse command history for the current session
- `Esc`: cancel the prompt
- `F1`: show key bindings

The prompt accepts commands with or without a leading colon.

## `sort`

Syntax:

```text
sort <field> [asc|desc] [field] [asc|desc] ...
```

Fields:

- `title`
- `authors`
- `series`
- `formats`
- `tags`

Directions:

- `asc`
- `desc`

If a direction is omitted, `asc` is used.

Examples:

```text
sort title asc
sort title desc
sort authors asc title asc
sort formats desc title asc
sort tags desc authors asc title asc
```

Sort keys are applied after search match-field priority from `layout.toml`. For example, if `title` is before `formats` in `layout.toml`, title matches are grouped before format-only matches; then the selected sort keys order items inside those groups.

## `help`

Show key bindings:

```text
help
```

This is equivalent to pressing `F1`.
