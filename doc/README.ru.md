# Calibre TUI

Небольшой терминальный интерфейс для поиска в библиотеке Calibre и открытия книг.

[English](../README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md)

### Возможности

* Поиск по названию, автору, серии и тегам. Термины через пробел объединяются логическим AND.
* Translators для Pinyin, японского, немецкого, французского, испанского и русского.
* Можно включить несколько translators одновременно. Поиск по исходному тексту всегда включен.
* Множественный выбор, открытие книг и вывод путей книг в stdout.
* Компактный фиксированный TUI без сложной настройки интерфейса или темы.

### Сборка

```bash
cargo build --release
./target/release/calibre-tui
```

### Настройка

В Linux файлы настройки находятся в `~/.config/calibre-tui/`. Если их нет, приложение создает файлы по умолчанию.

```toml
library_path = ""

[filter]
translators = ["pinyin", "romaji", "german-latin", "french-latin", "spanish-latin", "russian-latin"]
pinyin_fuzzy = true
pinyin_fuzzy_groups = [
    ["on", "ong"],
    ["an", "ang"],
    ["en", "eng"],
    ["in", "ing"]
]
```

* `library_path`: путь к библиотеке Calibre. Оставьте пустым для автоматического поиска в обычных местах.
* `filter.translators`: включенные search translators. Поддерживаются `pinyin`, `romaji`, `german-latin`, `french-latin`, `spanish-latin`, `russian-latin`.
* `filter.pinyin_fuzzy`: включает нестрогое сопоставление Pinyin.
* `filter.pinyin_fuzzy_groups`: эквивалентные фрагменты Pinyin. Первый элемент каждой группы считается канонической формой.

Поведение translators:

* `pinyin`: китайские Hanzi можно искать по Pinyin.
* `romaji`: kana можно искать по romaji. Полноширинный ASCII нормализуется. Произвольные чтения Kanji без словаря не угадываются, но поиск по исходному тексту работает.
* `german-latin`: `ä/ö/ü/ß` ищутся как `ae/oe/ue/ss`.
* `french-latin`: диакритика сворачивается, например `étranger` ищется как `etranger`.
* `spanish-latin`: диакритика сворачивается, например `niñez` ищется как `ninez`.
* `russian-latin`: кириллицу можно искать латинской транслитерацией, например `Преступление` как `prestuplenie`.

### Горячие клавиши

`keymap.toml` управляет горячими клавишами:

```toml
[browser]
keymap = [
  { on = "esc", run = "quit", desc = "Quit" },
  { on = "enter", run = "open", desc = "Open selected books" },
]

[global]
keymap = [
  { on = "f1", run = "help", desc = "Show key bindings" },
  { on = "ctrl-t", run = "command", desc = "Enter command" },
]
```

Актуальные имена клавиш, последовательности и действия описаны в [Keymap](keymap.md).

### Использование

* `Up` / `Down` или колесо мыши: переместить курсор.
* `PgUp` / `PgDown`: переместиться на страницу.
* `Home` / `End`: перейти к первому или последнему результату.
* `Tab`: переключить выбор текущей книги. При выборе курсор переходит на строку ниже.
* `Ctrl+A`: выбрать все текущие отфильтрованные результаты.
* `Ctrl+X`: очистить выбор.
* `Ctrl+P`: вывести выбранные пути в stdout и выйти.
* `Ctrl+S`, затем клавиша сортировки: применить частую сортировку.
* `Ctrl+T`: открыть режим команд.
* `F1`: показать справку по клавишам.
* `Enter`: открыть выбранные книги. Если ничего не выбрано, открыть книгу под курсором.
* `Esc` или `Ctrl+C`: выйти.
* `--exit-on-open`: выйти после открытия книг.

Актуальная подробная документация находится в [doc/index.md](index.md).
