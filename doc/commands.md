# Commands

Press `Ctrl+T` to open the command prompt, type a command, and press `Enter`. A leading `:` is optional. Completions appear as you type; `Tab` and `Shift+Tab` choose one and `Enter` accepts it (press `Enter` again to run). `Up` and `Down` recall commands from the current session.

## `sort`

```text
sort <field> [asc|desc] [<field> [asc|desc]] ...
```

Sorts the results by one or more fields. Later fields break ties in earlier ones.

- Fields: `title`, `authors`, `series`, `formats`, `tags` (also `name`, `author`, `format`, `tag`).
- Directions: `asc` (the default) or `desc` (also `ascending`, `descending`).
- Names are case-insensitive.

Examples:

```text
sort title desc
sort authors title
sort series asc title asc
sort formats desc title asc
```

The default order is `sort title asc`, and the current sort is shown in the search box title. Comparison ignores ASCII letter case. Multi-value fields compare as displayed, for example `Author A & Author B`.

While searching, books are first grouped by the searchable field they matched, following the column order in [`layout.toml`](layout.md): with the default layout, title matches come before author-only matches, and so on. The sort applies within each group. See [Search](search.md#result-order).

## `help`

```text
help
```

Shows the key bindings, like `F1`.
