# Calibre TUI

Небольшой терминальный интерфейс для поиска в библиотеке Calibre и открытия книг.

[English](README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md)

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
quit = ["esc", "ctrl-c"]
submit = ["enter"]
move_up = ["up"]
move_down = ["down"]
page_up = ["pgup"]
page_down = ["pgdown"]
jump_start = ["home"]
jump_end = ["end"]
toggle_selection = ["tab"]
select_all = ["ctrl-a"]
clear_selection = ["ctrl-x"]
delete_input = ["backspace"]
```

Имена клавиш поддерживают `enter`, `esc`, `tab`, `backspace`, `up`, `down`, `home`, `end`, `page-up`, `page-down`, одиночные символы и модификаторы вроде `ctrl-a`, `alt-x`, `shift-tab`. Также поддерживаются последовательности, например `jump_start = ["home", "ctrl-g g"]`.

### Использование

* `Up` / `Down` или колесо мыши: переместить курсор.
* `PgUp` / `PgDown`: переместиться на страницу.
* `Home` / `End`: перейти к первому или последнему результату.
* `Tab`: переключить выбор текущей книги. При выборе курсор переходит на строку ниже.
* `Ctrl+A`: выбрать все текущие отфильтрованные результаты.
* `Ctrl+X`: очистить выбор.
* `Enter`: открыть выбранные книги. Если ничего не выбрано, открыть книгу под курсором.
* `Esc` или `Ctrl+C`: выйти.
* `--exit-on-submit`: выйти после отправки.
* `--print-path`: вывести пути выбранных книг в stdout вместо открытия.
