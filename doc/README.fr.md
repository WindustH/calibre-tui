# Calibre TUI

Une interface en terminal pour chercher dans votre bibliothèque Calibre, puis ouvrir des livres ou afficher leurs chemins pour vos scripts shell.

[English](../README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Español](README.es.md) | [Русский](README.ru.md)

https://github.com/user-attachments/assets/7e741b94-80e0-4c61-8479-57e963c01d3e

## Fonctionnalités

- Recherche instantanée pendant la frappe dans le titre, les auteurs, la série, les formats et les étiquettes. Tous les termes séparés par des espaces doivent correspondre.
- Recherche d'autres écritures depuis un clavier latin : le chinois en pinyin (activé par défaut), les kana japonais en romaji, le russe par translittération, et l'allemand, le français ou l'espagnol sans accents.
- Sélection de plusieurs livres pour les ouvrir, copier leurs chemins, ou afficher les chemins et quitter pour les utiliser dans un pipeline shell.
- Choix d'un programme d'ouverture par format, par exemple `zathura` pour les PDF ; sinon, l'application par défaut du système est utilisée.
- Tri sur n'importe quel champ avec les raccourcis `Ctrl+S` ou la commande `sort`.
- Colonnes, raccourcis clavier (y compris les séquences de touches avec aide contextuelle) et couleurs personnalisables dans des fichiers TOML commentés.
- Fonctionne pendant que Calibre est ouvert ; la bibliothèque n'est jamais modifiée, seulement lue.

## Installation

Arch Linux (AUR) :

```bash
yay -S calibre-tui-bin   # binaire précompilé
yay -S calibre-tui       # dernière version, compilée depuis les sources
yay -S calibre-tui-git   # version git la plus récente, compilée depuis les sources
```

Homebrew :

```bash
brew install WindustH/tap/calibre-tui          # binaire précompilé
brew install --HEAD WindustH/tap/calibre-tui   # version git la plus récente
```

Des binaires précompilés pour Linux (x86_64), macOS (Apple Silicon) et Windows (x86_64) sont joints à chaque release GitHub.

Pour compiler depuis les sources, il faut Rust et, sous Linux, le paquet de développement SQLite (par exemple `libsqlite3-dev`) :

```bash
git clone --recursive https://github.com/WindustH/calibre-tui.git
cd calibre-tui
cargo build --release
./target/release/calibre-tui
```

## Utilisation

Lancez `calibre-tui`. La bibliothèque Calibre est détectée automatiquement ; si ce n'est pas le cas, renseignez `library_path` dans `config.toml`.

- Tapez pour chercher ; `Backspace` efface.
- `Up` / `Down` ou la molette : se déplacer ; `PgUp` / `PgDn`, `Home` / `End` : sauter.
- `Tab` : sélectionner ou désélectionner le livre courant. `Ctrl+A` sélectionne tous les résultats, `Ctrl+X` vide la sélection.
- `Enter` : ouvrir les livres sélectionnés, ou le livre courant si aucun n'est sélectionné.
- `Ctrl+Y` : copier leurs chemins dans le presse-papiers.
- `Ctrl+P` : afficher leurs chemins et quitter.
- `Ctrl+S`, puis une lettre : trier (`t` titre, `a` auteurs, `s` série, `f` formats, `g` étiquettes ; majuscule pour l'ordre décroissant).
- `Ctrl+T` : invite de commande, par exemple `sort authors asc title desc`.
- `F1` : afficher tous les raccourcis.
- `Esc` ou `Ctrl+C` : quitter.

Avec `--exit-on-open`, le programme se ferme après avoir ouvert des livres.

Les chemins sont écrits sur stdout, un par ligne, tandis que l'interface reste affichée dans le terminal ; ils sont donc utilisables dans des scripts :

```bash
zathura "$(calibre-tui)"                    # ouvrir un livre
calibre-tui | xargs -d '\n' -r cp -t ~/usb  # copier les livres sélectionnés (GNU xargs)
```

## Configuration

Au premier lancement, quatre fichiers sont créés avec des valeurs par défaut commentées :

- `config.toml` : chemin de la bibliothèque, programmes d'ouverture par format, translittérations de recherche
- `layout.toml` : colonnes, leur ordre et leur largeur, et champs utilisés par la recherche
- `keymap.toml` : raccourcis clavier
- `theme.toml` : couleurs

Ils se trouvent dans `~/.config/calibre-tui/` sous Linux, `~/Library/Application Support/calibre-tui/` sous macOS et `%APPDATA%\calibre-tui\` sous Windows. Quand une mise à jour ajoute des réglages, ils sont complétés avec leurs valeurs par défaut. Un fichier devenu illisible est sauvegardé sous `<nom>.bak-<horodatage>` et remplacé par les valeurs par défaut.

## Documentation

La documentation complète est en anglais :

- [Démarrage rapide](quick-start.md)
- [Commandes clavier](controls.md) et [invite de commande](commands.md)
- [Recherche](search.md)
- [Configuration](configuration.md), [disposition](layout.md), [raccourcis](keymap.md), [thème](theme.md)
- [Dépannage](troubleshooting.md)
- [Architecture](architecture.md), pour les contributeurs
