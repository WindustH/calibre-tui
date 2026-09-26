# Troubleshooting

## No Calibre library found

Startup fails with `library_path is empty and no Calibre library was found` when automatic detection finds nothing (see [Quick Start](quick-start.md#finding-your-library)). Set the path in `config.toml` ([where it is](configuration.md)):

```toml
library_path = "/home/me/Calibre Library"
```

## Invalid library path

`invalid Calibre library path '...': metadata.db was not found` means `library_path` points to a directory without `metadata.db`. The file is not reset in this case, since the library might be on a drive that isn't mounted. Fix the path or mount the drive.

## Calibre is running

That's fine: the database is opened read-only. If Calibre is in the middle of writing, calibre-tui waits briefly and then reads the file without locking, so the list may miss the very latest change. Restart calibre-tui to pick up new books.

## A config file was replaced

If a file couldn't be read (a typo, an unknown setting, a value of the wrong type), it was renamed to `<name>.bak-<timestamp>` and replaced with defaults; the message printed at startup names the backup and the error. Copy your settings back from the backup, fixing the reported problem.

## My comments in a config file disappeared

Files are only rewritten when settings are missing, typically after an upgrade adds new ones. The rewritten file keeps your values but uses the generated layout and comments. A file that already has every setting is never modified.

## A book doesn't open

- "has no book file": the book is a metadata-only entry in Calibre with no format attached.
- "book file not found": Calibre's database lists a file that is missing on disk. Check the library with Calibre.
- "failed to run open command": the program in `open.commands` couldn't be started; check its name and that it is on your `PATH`.
- Nothing happens: the system opener has no application for that format. Try `xdg-open <file>` (Linux) or `open <file>` (macOS) to see the error, or set a command in `open.commands`.
- The wrong program opens: set a per-format command in `config.toml`. Formats are matched by file extension, case-insensitively.

```toml
[open.commands]
pdf = ["zathura", "{path}"]
```

When a book has several formats, the most recently added one is opened.

## Copying paths fails

`Ctrl+Y` uses `wl-copy` (Wayland) or `xclip` / `xsel` (X11) on Linux, `pbcopy` on macOS, and `clip` on Windows. On Linux, install one of them.

## Using printed paths in scripts

When stdout is redirected, as in `$(calibre-tui)` or `calibre-tui | ...`, the interface is drawn on stderr and stdout receives only the paths printed by `Ctrl+P`. Don't redirect stderr as well, or the interface won't be visible.

## A shortcut doesn't work

- Press `F1` to see which bindings are in effect.
- The terminal may not report the combination: `Ctrl+/` and `Ctrl+;` in particular are unreliable, which is why the command prompt uses `Ctrl+T`. Bind another key in [`keymap.toml`](keymap.md).
- While the footer shows which-key hints, the next key continues the sequence. `Esc` cancels it; a second `Esc` quits.

## `Enter` in the command prompt doesn't run the command

While a completion is highlighted, `Enter` accepts it first. Press `Enter` again to run the command.
