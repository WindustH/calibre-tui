# Calibre TUI

Una pequeña interfaz de terminal para buscar en una biblioteca de Calibre y abrir libros.

[English](../README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Русский](README.ru.md)

### Funciones

* Busca por título, autor, serie y etiquetas. Los términos separados por espacios se combinan con AND lógico.
* Translators para Pinyin, japonés, alemán, francés, español y ruso.
* Se pueden activar varios translators al mismo tiempo. La búsqueda por texto original siempre está activa.
* Selección múltiple, apertura directa y salida de rutas de libros por stdout.
* TUI compacta y fija, sin configuración compleja de UI o temas.

### Compilación

```bash
cargo build --release
./target/release/calibre-tui
```

### Configuración

En Linux, los archivos de configuración están en `~/.config/calibre-tui/`. Si no existen, la aplicación escribe archivos predeterminados.

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

* `library_path`: ruta a la biblioteca de Calibre. Déjala vacía para detectar ubicaciones comunes.
* `filter.translators`: translators de búsqueda a activar. Soporta `pinyin`, `romaji`, `german-latin`, `french-latin`, `spanish-latin` y `russian-latin`.
* `filter.pinyin_fuzzy`: activa coincidencias Pinyin aproximadas.
* `filter.pinyin_fuzzy_groups`: fragmentos Pinyin equivalentes. El primer elemento de cada grupo es la forma canónica.

Comportamiento de translators:

* `pinyin`: los Hanzi chinos se pueden buscar por Pinyin.
* `romaji`: los kana se pueden buscar por romaji. El ASCII de ancho completo se normaliza. Las lecturas arbitrarias de Kanji no se infieren sin diccionario, pero la búsqueda por texto original sigue funcionando.
* `german-latin`: `ä/ö/ü/ß` se pueden encontrar como `ae/oe/ue/ss`.
* `french-latin`: los acentos se pliegan, por ejemplo `étranger` se puede encontrar con `etranger`.
* `spanish-latin`: los acentos se pliegan, por ejemplo `niñez` se puede encontrar con `ninez`.
* `russian-latin`: el cirílico se puede buscar por transliteración latina, por ejemplo `Преступление` con `prestuplenie`.

### Atajos

`keymap.toml` controla los atajos:

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

Los nombres de teclas soportan `enter`, `esc`, `tab`, `backspace`, `up`, `down`, `home`, `end`, `page-up`, `page-down`, caracteres simples y modificadores como `ctrl-a`, `alt-x` o `shift-tab`. También se admiten secuencias como `jump_start = ["home", "ctrl-g g"]`.

### Uso

* `Up` / `Down` o rueda del mouse: mover el cursor.
* `PgUp` / `PgDown`: mover una página.
* `Home` / `End`: saltar al primer o último resultado.
* `Tab`: alternar la selección del libro actual. Al seleccionarlo, el cursor baja una fila.
* `Ctrl+A`: seleccionar todos los resultados filtrados.
* `Ctrl+X`: limpiar la selección.
* `Enter`: abrir los libros seleccionados. Si no hay selección, abrir el libro bajo el cursor.
* `Esc` o `Ctrl+C`: salir.
* `--exit-on-submit`: salir después de enviar.
* `--print-path`: imprimir las rutas de libros en stdout sin abrirlos.
