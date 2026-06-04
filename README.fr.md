# Calibre TUI

Une petite interface terminal pour rechercher dans une bibliothèque Calibre et ouvrir des livres.

[English](README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Русский](README.ru.md)

### Fonctionnalités

* Recherche par titre, auteur, série et étiquettes. Les termes séparés par des espaces sont combinés avec un AND logique.
* Translators pour Pinyin, japonais, allemand, français, espagnol et russe.
* Plusieurs translators peuvent être activés en même temps. La recherche en texte original est toujours active.
* Sélection multiple, ouverture directe et sortie des chemins de livres vers stdout.
* Interface TUI compacte et fixe, sans configuration avancée d'interface ou de thème.

### Compilation

```bash
cargo build --release
./target/release/calibre-tui
```

### Configuration

Sous Linux, les fichiers de configuration sont dans `~/.config/calibre-tui/`. S'ils n'existent pas, l'application écrit les fichiers par défaut.

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

* `library_path` : chemin vers la bibliothèque Calibre. Laissez vide pour détecter les emplacements courants.
* `filter.translators` : translators de recherche à activer. Valeurs prises en charge : `pinyin`, `romaji`, `german-latin`, `french-latin`, `spanish-latin`, `russian-latin`.
* `filter.pinyin_fuzzy` : active la correspondance Pinyin approximative.
* `filter.pinyin_fuzzy_groups` : fragments Pinyin équivalents. Le premier élément de chaque groupe est la forme canonique.

Comportement des translators :

* `pinyin` : les Hanzi chinois peuvent être recherchés en Pinyin.
* `romaji` : les kana peuvent être recherchés en romaji. L'ASCII pleine chasse est normalisé. Les lectures arbitraires des Kanji ne sont pas déduites sans dictionnaire, mais la recherche en texte original reste active.
* `german-latin` : `ä/ö/ü/ß` peuvent être trouvés avec `ae/oe/ue/ss`.
* `french-latin` : les accents sont repliés, par exemple `étranger` peut être trouvé avec `etranger`.
* `spanish-latin` : les accents sont repliés, par exemple `niñez` peut être trouvé avec `ninez`.
* `russian-latin` : le cyrillique peut être recherché par translittération latine, par exemple `Преступление` avec `prestuplenie`.

### Raccourcis

`keymap.toml` contrôle les raccourcis :

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

Les noms de touches prennent en charge `enter`, `esc`, `tab`, `backspace`, `up`, `down`, `home`, `end`, `page-up`, `page-down`, les caractères simples et les modificateurs comme `ctrl-a`, `alt-x` ou `shift-tab`. Les séquences comme `jump_start = ["home", "ctrl-g g"]` sont aussi prises en charge.

### Utilisation

* `Up` / `Down` ou molette : déplacer le curseur.
* `PgUp` / `PgDown` : déplacer d'une page.
* `Home` / `End` : aller au premier ou au dernier résultat.
* `Tab` : basculer la sélection du livre courant. En sélectionnant, le curseur descend d'une ligne.
* `Ctrl+A` : sélectionner tous les résultats filtrés.
* `Ctrl+X` : vider la sélection.
* `Enter` : ouvrir les livres sélectionnés. Si rien n'est sélectionné, ouvrir le livre sous le curseur.
* `Esc` ou `Ctrl+C` : quitter.
* `--exit-on-submit` : quitter après validation.
* `--print-path` : écrire les chemins des livres dans stdout sans les ouvrir.
