# Calibre TUI

Calibre ライブラリをターミナルで検索し、本を開いたり、シェルスクリプト用にファイルパスを出力したりできる TUI です。

[English](../README.md) | [中文](README.zh-CN.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Español](README.es.md) | [Русский](README.ru.md)

https://github.com/user-attachments/assets/7e741b94-80e0-4c61-8479-57e963c01d3e

## 特長

- 入力と同時に、タイトル・著者・シリーズ・形式・タグを検索します。スペースで区切った語はすべて一致する必要があります。
- ラテン文字のキーボードで他の文字を検索できます。中国語はピンイン（既定で有効）、日本語の仮名はローマ字、ロシア語は翻字で検索でき、ドイツ語・フランス語・スペイン語はアクセントなしで入力できます。
- 複数の本を選択して開く、パスをコピーする、またはパスを出力して終了し、シェルのパイプで使うことができます。
- 形式ごとに開くプログラムを指定できます（例：PDF は `zathura`）。指定がない形式はシステムの既定アプリで開きます。
- `Ctrl+S` のショートカットや `sort` コマンドで任意のフィールドで並べ替えられます。
- 列、キー割り当て（ヒント付きの複数キーシーケンスを含む）、配色を、コメント付きの TOML ファイルで設定できます。
- Calibre の起動中でも使えます。ライブラリは読み取るだけで、書き込みは一切しません。

## インストール

Arch Linux（AUR）：

```bash
yay -S calibre-tui-bin   # ビルド済みバイナリ
yay -S calibre-tui       # 最新リリースをソースからビルド
yay -S calibre-tui-git   # 最新の git 版をソースからビルド
```

Homebrew：

```bash
brew install WindustH/tap/calibre-tui          # ビルド済みバイナリ
brew install --HEAD WindustH/tap/calibre-tui   # 最新の git 版
```

各 GitHub リリースには、Linux（x86_64）、macOS（Apple Silicon）、Windows（x86_64）向けのビルド済みバイナリが添付されています。

ソースからビルドするには Rust が必要です。Linux では SQLite の開発パッケージ（例：`libsqlite3-dev`）も必要です。

```bash
git clone --recursive https://github.com/WindustH/calibre-tui.git
cd calibre-tui
cargo build --release
./target/release/calibre-tui
```

## 使い方

`calibre-tui` を実行します。Calibre ライブラリは自動で見つかります。見つからない場合は `config.toml` の `library_path` を設定してください。

- 文字を入力すると検索、`Backspace` で削除します。
- `Up` / `Down` またはマウスホイール：移動。`PgUp` / `PgDn`、`Home` / `End`：ジャンプ。
- `Tab`：フォーカス中の本を選択／選択解除。`Ctrl+A` で結果をすべて選択、`Ctrl+X` で選択を解除。
- `Enter`：選択した本を開きます。選択がなければフォーカス中の本を開きます。
- `Ctrl+Y`：それらのパスをクリップボードにコピー。
- `Ctrl+P`：それらのパスを出力して終了。
- `Ctrl+S` の後に文字キー：並べ替え（`t` タイトル、`a` 著者、`s` シリーズ、`f` 形式、`g` タグ。大文字で降順）。
- `Ctrl+T`：コマンドプロンプト（例：`sort authors asc title desc`）。
- `F1`：すべてのキー割り当てを表示。
- `Esc` または `Ctrl+C`：終了。

`--exit-on-open` を付けると、本を開いた後に終了します。

パスは 1 行に 1 つずつ stdout に出力され、画面はターミナルに表示されたままなので、スクリプトで利用できます。

```bash
zathura "$(calibre-tui)"                    # 1 冊を開く
calibre-tui | xargs -d '\n' -r cp -t ~/usb  # 選択した本をコピー（GNU xargs）
```

## 設定

初回起動時に、コメント付きの既定値で 4 つのファイルが作成されます。

- `config.toml`：ライブラリのパス、形式ごとの起動プログラム、検索用の変換
- `layout.toml`：表示する列、その順序と幅、検索対象のフィールド
- `keymap.toml`：キー割り当て
- `theme.toml`：配色

場所は Linux が `~/.config/calibre-tui/`、macOS が `~/Library/Application Support/calibre-tui/`、Windows が `%APPDATA%\calibre-tui\` です。アップデートで設定項目が増えると、既定値で自動的に補われます。読み込めなくなったファイルは `<ファイル名>.bak-<タイムスタンプ>` として保存され、既定値に置き換えられます。

## ドキュメント

詳しいドキュメントは英語です。

- [クイックスタート](quick-start.md)
- [操作](controls.md) と [コマンド](commands.md)
- [検索](search.md)
- [設定](configuration.md)、[レイアウト](layout.md)、[キーマップ](keymap.md)、[テーマ](theme.md)
- [トラブルシューティング](troubleshooting.md)
- [アーキテクチャ](architecture.md)（開発者向け）
