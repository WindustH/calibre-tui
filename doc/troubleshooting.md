# Troubleshooting

## No Calibre Library Found

Set `library_path` in `~/.config/calibre-tui/config.toml`:

```toml
library_path = "/home/me/Calibre Library"
```

The directory must contain `metadata.db`.

If `library_path` is empty, the app tries common locations. If none contain `metadata.db`, startup fails with a clear error.

## Invalid Library Path

An explicit invalid `library_path` is not treated as an incompatible config file. The app reports the error instead of overwriting your config, because the path may be temporarily unavailable due to an unmounted disk.

## Config Was Replaced

When a config file cannot be parsed or no longer matches the expected structure, it is backed up before a new default file is generated:

```text
theme.toml.bak-<timestamp>
```

Review the backup and copy over any values you still want.

## New Config Fields Keep Appearing

When a version adds fields, the app fills missing values with defaults and rewrites the file with comments. This is expected. Existing values are preserved.

Repeated item fields are not documented repeatedly. For example, `columns.field` is documented on the first `[[columns]]` item only.

Short scalar arrays are automatically rewritten onto one line when they fit. Longer arrays and arrays containing nested items stay multi-line.

## A Format Opens With The Wrong App

Set a format-specific command in `~/.config/calibre-tui/config.toml`:

```toml
[open.commands]
pdf = ["zathura", "{path}"]
epub = ["foliate", "{path}"]
```

The command is executed as argv, not through a shell. If `{path}` is omitted, the file path is appended as the final argument. Formats are matched by file extension, case-insensitively.

## `Ctrl+/` Or `Ctrl+;` Does Not Open Commands

The default command shortcut is `Ctrl+T`. Some terminals report `Ctrl+/` and `Ctrl+;` inconsistently, so they are not used by default.

Change `global.keymap` in `keymap.toml` if you prefer a different shortcut.

## Which-Key Does Not Go Away

When a multi-key sequence is pending, press `Esc` to cancel the pending sequence. A second `Esc` quits the app in the default browser context.

## Command Completion Does Not Run Immediately

If a completion candidate is active, `Enter` applies it first. Press `Enter` again to run the completed command. When the current token is already a complete unique candidate, completion is cleared so the command can run directly.
