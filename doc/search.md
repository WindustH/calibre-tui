# Search

`calibre-tui` builds an in-memory search index from Calibre metadata and searches against the fields enabled in `layout.toml`.

## Search Terms

Space-separated terms are matched with logical AND. A book must match every term to appear in the result list.

Searchable fields are controlled by `layout.toml`:

```toml
[[columns]]
field = "title"
search = true

[[columns]]
field = "formats"
search = true
```

Matching text is highlighted in visible fields.

## Search Fields

Supported fields:

- `title`
- `authors`
- `series`
- `formats`
- `tags`

You can search a field without showing it by setting `visible = false` and `search = true`.

## Result Ordering

Result ordering has two phases:

1. Match-field priority from `layout.toml`
2. Explicit sort keys from `Ctrl+S` shortcuts or the `sort` command

The first searchable field that matched determines the primary group. If `title` is before `tags`, title matches are listed before tag-only matches. Explicit sort keys then order books inside those groups.

## Translators

Translators are enabled in `config.toml`:

```toml
[filter]
translators = ["pinyin", "romaji", "german-latin"]
```

Supported translators:

- `pinyin`: Chinese Hanzi can be searched by pinyin. Optional fuzzy groups can treat fragments as equivalent.
- `romaji`: Japanese kana can be searched by romaji. Full-width ASCII is normalized. Arbitrary kanji readings are not inferred without a dictionary, but original text search still works.
- `german-latin`: German accented Latin folding, including ae/oe/ue/ss-style matching.
- `french-latin`: French accented Latin folding.
- `spanish-latin`: Spanish accented Latin folding.
- `russian-latin`: Cyrillic-to-Latin transliteration support.

Original text search is always enabled even when translators are configured.
