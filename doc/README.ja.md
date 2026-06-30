# Calibre TUI

Calibre ライブラリを端末上で検索し、本を開くための小さな TUI です。

[English](../README.md) | [中文](README.zh-CN.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Русский](README.ru.md)

### 機能

* タイトル、著者、シリーズ、タグを検索できます。空白で区切った語は AND 条件で一致します。
* Pinyin、日本語、ドイツ語、フランス語、スペイン語、ロシア語の translator をサポートします。
* 複数の translator を同時に有効化できます。原文検索は常に有効です。
* 複数選択、直接オープン、stdout へのパス出力に対応します。
* UI とテーマの細かい設定は持たず、固定でコンパクトな画面です。

### ビルド

```bash
cargo build --release
./target/release/calibre-tui
```

### 設定

Linux では設定ファイルは `~/.config/calibre-tui/` に保存されます。存在しない場合は既定値が生成されます。

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

* `library_path`: Calibre ライブラリのパスです。空の場合は一般的な場所から自動検出します。
* `filter.translators`: 有効にする検索 translator です。`pinyin`、`romaji`、`german-latin`、`french-latin`、`spanish-latin`、`russian-latin` を指定できます。
* `filter.pinyin_fuzzy`: Pinyin のあいまい一致を有効にします。
* `filter.pinyin_fuzzy_groups`: 等価な Pinyin 断片です。各グループの先頭が正規形です。

Translator の動作:

* `pinyin`: 中国語の漢字を Pinyin で検索できます。
* `romaji`: かなを romaji で検索できます。全角 ASCII も正規化します。辞書なしで任意の漢字読みは推定しませんが、原文検索は使えます。
* `german-latin`: `ä/ö/ü/ß` を `ae/oe/ue/ss` として検索できます。
* `french-latin`: アクセント付きラテン文字を折りたたみ、`étranger` を `etranger` で検索できます。
* `spanish-latin`: アクセント付きラテン文字を折りたたみ、`niñez` を `ninez` で検索できます。
* `russian-latin`: キリル文字をラテン転写で検索できます。例: `Преступление` は `prestuplenie`。

### キー設定

`keymap.toml` でショートカットを変更できます。

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

キー名は `enter`、`esc`、`tab`、`backspace`、`up`、`down`、`home`、`end`、`page-up`、`page-down`、単一文字、`ctrl-a`、`alt-x`、`shift-tab` などに対応します。`jump_start = ["home", "ctrl-g g"]` のようなキーシーケンスも使えます。

### 使い方

* `Up` / `Down` またはマウスホイール: カーソル移動。
* `PgUp` / `PgDown`: 1 ページ移動。
* `Home` / `End`: 先頭または末尾へ移動。
* `Tab`: 現在の本の選択を切り替えます。選択した場合は次の行へ移動します。
* `Ctrl+A`: 現在の検索結果をすべて選択。
* `Ctrl+X`: 選択をすべて解除。
* `Enter`: 選択中の本を開きます。選択がない場合はカーソル上の本を開きます。
* `Esc` または `Ctrl+C`: 終了。
* `--exit-on-submit`: 送信後に終了。
* `--print-path`: 本を開かず、選択した本のパスを stdout に出力。
