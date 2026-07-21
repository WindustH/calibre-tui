# Controls

## Browser

Default browser controls:

- Type text: append to the search query.
- `Backspace`: delete the last search character.
- `Up` / `Down`: move focus.
- Mouse wheel: move focus.
- `PgUp` / `PgDown`: move by one page.
- `Home` / `End`: jump to the first or last result.
- `Tab`: toggle selection for the focused book and move to the next row.
- `Ctrl+A`: select all current results.
- `Ctrl+X`: clear selection.
- `Enter`: open selected books, or the focused book if nothing is selected.
- `Ctrl+P`: print selected/focused paths to stdout and quit.
- `Ctrl+S` plus a follow-up key: apply a common sort.
- `Ctrl+T`: open the command prompt.
- `F1`: show key bindings.
- `Esc` / `Ctrl+C`: quit.

## Sort Prefix

Default `Ctrl+S` follow-up keys:

- `t`: sort title ascending
- `T`: sort title descending
- `a`: sort authors ascending
- `A`: sort authors descending
- `s`: sort series ascending
- `S`: sort series descending
- `f`: sort formats ascending
- `F`: sort formats descending
- `g`: sort tags ascending
- `G`: sort tags descending

When which-key is waiting for a follow-up key, `Esc` cancels the waiting state first instead of quitting the app.

## Command Prompt

Default prompt controls:

- `Esc`: cancel
- `Enter`: submit or apply active completion
- `Backspace` / `Delete`: delete around cursor
- `Left` / `Right`: move cursor
- `Home` / `End`: move to start/end
- `Ctrl+A` / `Ctrl+E`: move to start/end
- `Ctrl+U`: delete before cursor
- `Ctrl+K`: delete after cursor
- `Tab` / `Shift+Tab`: cycle completion candidates
- `Up` / `Down`: browse command history

## F1 Help

`F1` opens the key binding help dialog for the current context. Close it with `Esc`, `q`, `Enter`, or `F1`.
