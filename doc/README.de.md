# Calibre TUI

Eine kleine Terminal-Oberfläche zum Durchsuchen einer Calibre-Bibliothek und Öffnen von Büchern.

[English](../README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Español](README.es.md) | [Русский](README.ru.md)

### Funktionen

* Suche nach Titel, Autor, Reihe und Tags. Durch Leerzeichen getrennte Begriffe werden mit logischem AND verknüpft.
* Translator für Pinyin, Japanisch, Deutsch, Französisch, Spanisch und Russisch.
* Mehrere Translator können gleichzeitig aktiviert werden. Originaltextsuche ist immer aktiv.
* Mehrfachauswahl, direktes Öffnen und Ausgabe der Buchpfade auf stdout.
* Kompaktes festes TUI ohne umfangreiche UI- oder Theme-Konfiguration.

### Build

```bash
cargo build --release
./target/release/calibre-tui
```

### Konfiguration

Unter Linux liegen die Konfigurationsdateien in `~/.config/calibre-tui/`. Falls sie fehlen, schreibt das Programm Standarddateien.

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

* `library_path`: Pfad zur Calibre-Bibliothek. Leer lassen, um übliche Orte automatisch zu erkennen.
* `filter.translators`: Aktivierte Such-Translator. Unterstützt werden `pinyin`, `romaji`, `german-latin`, `french-latin`, `spanish-latin` und `russian-latin`.
* `filter.pinyin_fuzzy`: Aktiviert unscharfes Pinyin-Matching.
* `filter.pinyin_fuzzy_groups`: Gleichwertige Pinyin-Fragmente. Der erste Eintrag jeder Gruppe ist die kanonische Form.

Translator-Verhalten:

* `pinyin`: Chinesische Hanzi können per Pinyin gesucht werden.
* `romaji`: Kana können per romaji gesucht werden. Vollbreite ASCII-Zeichen werden normalisiert. Beliebige Kanji-Lesungen werden ohne Wörterbuch nicht erraten, Originaltextsuche funktioniert aber weiter.
* `german-latin`: `ä/ö/ü/ß` sind als `ae/oe/ue/ss` auffindbar.
* `french-latin`: Akzentzeichen werden gefaltet, zum Beispiel `étranger` zu `etranger`.
* `spanish-latin`: Akzentzeichen werden gefaltet, zum Beispiel `niñez` zu `ninez`.
* `russian-latin`: Kyrillisch kann per lateinischer Umschrift gesucht werden, zum Beispiel `Преступление` mit `prestuplenie`.

### Tastenkürzel

`keymap.toml` steuert die Tastenkürzel:

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

Aktuelle Tastennamen, Sequenzen und Aktionen stehen in [Keymap](keymap.md).

### Nutzung

* `Up` / `Down` oder Mausrad: Cursor bewegen.
* `PgUp` / `PgDown`: Eine Seite bewegen.
* `Home` / `End`: Zum ersten oder letzten Ergebnis springen.
* `Tab`: Auswahl des aktuellen Buchs umschalten. Beim Auswählen springt der Cursor eine Zeile weiter.
* `Ctrl+A`: Alle aktuellen Suchergebnisse auswählen.
* `Ctrl+X`: Auswahl leeren.
* `Ctrl+P`: Ausgewählte Buchpfade nach stdout ausgeben und beenden.
* `Ctrl+S` gefolgt von einer Sortiertaste: Eine Standardsortierung anwenden.
* `Ctrl+T`: Befehlsmodus öffnen.
* `F1`: Tastenhilfe anzeigen.
* `Enter`: Ausgewählte Bücher öffnen. Wenn nichts ausgewählt ist, wird das Buch unter dem Cursor geöffnet.
* `Esc` oder `Ctrl+C`: Beenden.
* `--exit-on-open`: Nach dem Öffnen der Bücher beenden.

Die aktuelle ausführliche Dokumentation steht unter [doc/index.md](index.md).
