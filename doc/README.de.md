# Calibre TUI

Eine Terminal-Oberfläche, um die eigene Calibre-Bibliothek zu durchsuchen und Bücher zu öffnen oder ihre Pfade für Shell-Skripte auszugeben.

[English](../README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Español](README.es.md) | [Русский](README.ru.md)

https://github.com/user-attachments/assets/7e741b94-80e0-4c61-8479-57e963c01d3e

## Funktionen

- Sofortige Suche während der Eingabe in Titel, Autoren, Reihe, Formaten und Schlagwörtern. Durch Leerzeichen getrennte Begriffe müssen alle zutreffen.
- Andere Schriften über eine lateinische Tastatur finden: Chinesisch per Pinyin (standardmäßig aktiv), japanische Kana per Romaji, Russisch per Transliteration sowie Deutsch, Französisch oder Spanisch ohne Akzente und Umlaute.
- Mehrere Bücher auswählen und öffnen, ihre Pfade kopieren oder die Pfade ausgeben und beenden, zur Verwendung in Shell-Pipelines.
- Pro Format ein eigenes Programm zum Öffnen festlegen, etwa `zathura` für PDF; sonst wird die Standardanwendung des Systems verwendet.
- Nach jedem Feld sortieren, per `Ctrl+S`-Tastenkürzel oder mit dem Befehl `sort`.
- Spalten, Tastenbelegung (auch Tastenfolgen mit Hinweisen) und Farben in kommentierten TOML-Dateien anpassen.
- Funktioniert, während Calibre läuft; die Bibliothek wird ausschließlich gelesen.

## Installation

Arch Linux (AUR):

```bash
yay -S calibre-tui-bin   # vorkompilierte Binärdatei
yay -S calibre-tui       # neueste Version, aus dem Quellcode gebaut
yay -S calibre-tui-git   # aktueller Git-Stand, aus dem Quellcode gebaut
```

Homebrew:

```bash
brew install WindustH/tap/calibre-tui          # vorkompilierte Binärdatei
brew install --HEAD WindustH/tap/calibre-tui   # aktueller Git-Stand
```

Jedem GitHub-Release liegen vorkompilierte Binärdateien für Linux (x86_64), macOS (Apple Silicon) und Windows (x86_64) bei.

Zum Bauen aus dem Quellcode wird Rust benötigt, unter Linux außerdem das SQLite-Entwicklungspaket (zum Beispiel `libsqlite3-dev`):

```bash
git clone --recursive https://github.com/WindustH/calibre-tui.git
cd calibre-tui
cargo build --release
./target/release/calibre-tui
```

## Verwendung

Starte `calibre-tui`. Die Calibre-Bibliothek wird automatisch gefunden; falls nicht, trage `library_path` in `config.toml` ein.

- Tippen sucht, `Backspace` löscht.
- `Up` / `Down` oder Mausrad: bewegen; `PgUp` / `PgDn`, `Home` / `End`: springen.
- `Tab`: das markierte Buch auswählen oder abwählen. `Ctrl+A` wählt alle Treffer aus, `Ctrl+X` hebt die Auswahl auf.
- `Enter`: die ausgewählten Bücher öffnen, oder das markierte, wenn nichts ausgewählt ist.
- `Ctrl+Y`: ihre Pfade in die Zwischenablage kopieren.
- `Ctrl+P`: ihre Pfade ausgeben und beenden.
- `Ctrl+S`, dann ein Buchstabe: sortieren (`t` Titel, `a` Autoren, `s` Reihe, `f` Formate, `g` Schlagwörter; Großbuchstabe für absteigend).
- `Ctrl+T`: Befehlszeile, zum Beispiel `sort authors asc title desc`.
- `F1`: alle Tastenbelegungen anzeigen.
- `Esc` oder `Ctrl+C`: beenden.

Mit `--exit-on-open` beendet sich das Programm nach dem Öffnen von Büchern.

Ausgegebene Pfade landen zeilenweise auf stdout, während die Oberfläche im Terminal bleibt; so lassen sie sich in Skripten verwenden:

```bash
zathura "$(calibre-tui)"                    # ein Buch öffnen
calibre-tui | xargs -d '\n' -r cp -t ~/usb  # ausgewählte Bücher kopieren (GNU xargs)
```

## Konfiguration

Beim ersten Start werden vier Dateien mit kommentierten Standardwerten angelegt:

- `config.toml`: Bibliothekspfad, Programme pro Format, Such-Transliterationen
- `layout.toml`: Spalten, ihre Reihenfolge und Breite sowie die durchsuchten Felder
- `keymap.toml`: Tastenbelegung
- `theme.toml`: Farben

Sie liegen unter Linux in `~/.config/calibre-tui/`, unter macOS in `~/Library/Application Support/calibre-tui/` und unter Windows in `%APPDATA%\calibre-tui\`. Bringt ein Update neue Einstellungen mit, werden sie mit Standardwerten ergänzt. Eine Datei, die nicht mehr gelesen werden kann, wird als `<Name>.bak-<Zeitstempel>` gesichert und durch die Standardwerte ersetzt.

## Dokumentation

Die ausführliche Dokumentation ist auf Englisch:

- [Schnellstart](quick-start.md)
- [Bedienung](controls.md) und [Befehle](commands.md)
- [Suche](search.md)
- [Konfiguration](configuration.md), [Layout](layout.md), [Tastenbelegung](keymap.md), [Farbschema](theme.md)
- [Fehlerbehebung](troubleshooting.md)
- [Architektur](architecture.md), für Mitwirkende
