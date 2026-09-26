# Search

Typing in the book list filters it immediately. The search runs over an in-memory index built at startup, so it stays fast on large libraries.

## Matching

- The query is split on spaces into terms. A book is shown only if **every** term matches at least one searchable field; different terms may match different fields.
- Matching is a case-insensitive substring match. Spaces inside titles and names are ignored, so `harrypot` matches `Harry Potter`.
- Matches are highlighted in visible columns (the first match of each term per field).
- An empty query shows the whole library.

## Searchable Fields

`title`, `authors`, `series`, `formats`, and `tags` are all searched by default. In [`layout.toml`](layout.md), `search = false` excludes a field and `visible = false` hides a column while still searching it.

Authors are matched as displayed, joined with ` & `, and formats and tags as comma-separated lists, for example `EPUB, PDF`.

## Translators

Translators add alternative spellings to the index, so text in another script can be found from a Latin keyboard. The original text is always searched as well. Enable them in `config.toml`:

```toml
[filter]
translators = ["pinyin", "romaji", "russian-latin"]
```

| Translator | What it does | Example |
| --- | --- | --- |
| `pinyin` (default) | Chinese characters as toneless pinyin, written together. Polyphonic characters use their most common reading. | `zhongguo` or `guoli` finds `中国历史` |
| `romaji` | Hiragana and katakana as Hepburn romaji, including `kya`-style digraphs, doubled consonants for `っ`, and `ー` long vowels. Kanji are not converted. | `kyari` finds `きゃりー` |
| `russian-latin` | Cyrillic transliterated to Latin (`ж` → `zh`, `щ` → `shch`, `ё` → `yo`; `ъ` and `ь` dropped). | `prestuplenie` finds `Преступление` |
| `german-latin` | `ä` → `ae`, `ö` → `oe`, `ü` → `ue`, `ß` → `ss`, plus the accent folding below. | `fuer` finds `für` |
| `french-latin`, `spanish-latin` | Accents removed (`é` → `e`, `ñ` → `n`, `æ` → `ae`, `œ` → `oe`). The two are identical; enabling either is enough. | `etranger` finds `L'Étranger` |

All translators also turn full-width letters and digits (`Ｆ１`) into ASCII. `romaji` and the Latin/Cyrillic translators drop punctuation, so `letranger` also finds `L'Étranger`.

### Fuzzy Pinyin

With `pinyin_fuzzy = true` (the default), each group in `pinyin_fuzzy_groups` is treated as equivalent. The defaults make `on`/`ong`, `an`/`ang`, `en`/`eng`, and `in`/`ing` interchangeable, so `shanhai` finds `上海` (`shanghai`). The first entry of each group is the canonical form; other common groups are `["z", "zh"]`, `["c", "ch"]`, `["s", "sh"]`, and `["n", "l"]`.

## Result Order

Results are ordered in two steps:

1. **Match priority**: books are grouped by the first searchable field, in `layout.toml` column order, that any term matched. With the default layout, title matches come first, then books that matched only on authors, then series, formats, and tags.
2. **Sort**: within each group, books follow the current sort (`title asc` by default), set with `Ctrl+S` shortcuts or the [`sort` command](commands.md).

With an empty query only the sort applies.
