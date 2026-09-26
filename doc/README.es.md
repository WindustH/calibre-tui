# Calibre TUI

Una interfaz de terminal para buscar en tu biblioteca de Calibre y después abrir libros o imprimir sus rutas para scripts de shell.

[English](../README.md) | [中文](README.zh-CN.md) | [日本語](README.ja.md) | [Deutsch](README.de.md) | [Français](README.fr.md) | [Русский](README.ru.md)

https://github.com/user-attachments/assets/7e741b94-80e0-4c61-8479-57e963c01d3e

## Funciones

- Búsqueda instantánea mientras escribes en título, autores, serie, formatos y etiquetas. Todos los términos separados por espacios deben coincidir.
- Busca en otras escrituras desde un teclado latino: chino por pinyin (activado por defecto), kana japonés por romaji, ruso por transliteración, y alemán, francés o español sin tildes ni diéresis.
- Selecciona varios libros para abrirlos, copiar sus rutas, o imprimir las rutas y salir para usarlas en tuberías de shell.
- Elige un programa para abrir cada formato, por ejemplo `zathura` para PDF; si no, se usa la aplicación predeterminada del sistema.
- Ordena por cualquier campo con los atajos `Ctrl+S` o el comando `sort`.
- Personaliza columnas, atajos de teclado (incluidas secuencias de varias teclas con pistas) y colores en archivos TOML comentados.
- Funciona con Calibre abierto; la biblioteca solo se lee, nunca se modifica.

## Instalación

Arch Linux (AUR):

```bash
yay -S calibre-tui-bin   # binario precompilado
yay -S calibre-tui       # última versión, compilada desde el código fuente
yay -S calibre-tui-git   # última versión de git, compilada desde el código fuente
```

Homebrew:

```bash
brew install WindustH/tap/calibre-tui          # binario precompilado
brew install --HEAD WindustH/tap/calibre-tui   # última versión de git
```

Cada release de GitHub incluye binarios precompilados para Linux (x86_64), macOS (Apple Silicon) y Windows (x86_64).

Para compilar desde el código fuente necesitas Rust y, en Linux, el paquete de desarrollo de SQLite (por ejemplo `libsqlite3-dev`):

```bash
git clone --recursive https://github.com/WindustH/calibre-tui.git
cd calibre-tui
cargo build --release
./target/release/calibre-tui
```

## Uso

Ejecuta `calibre-tui`. La biblioteca de Calibre se detecta automáticamente; si no se encuentra, define `library_path` en `config.toml`.

- Escribe para buscar; `Backspace` borra.
- `Up` / `Down` o la rueda del ratón: moverse; `PgUp` / `PgDn`, `Home` / `End`: saltar.
- `Tab`: seleccionar o deseleccionar el libro actual. `Ctrl+A` selecciona todos los resultados y `Ctrl+X` borra la selección.
- `Enter`: abrir los libros seleccionados, o el libro actual si no hay ninguno seleccionado.
- `Ctrl+Y`: copiar sus rutas al portapapeles.
- `Ctrl+P`: imprimir sus rutas y salir.
- `Ctrl+S` y después una letra: ordenar (`t` título, `a` autores, `s` serie, `f` formatos, `g` etiquetas; en mayúscula, orden descendente).
- `Ctrl+T`: línea de comandos, por ejemplo `sort authors asc title desc`.
- `F1`: mostrar todos los atajos.
- `Esc` o `Ctrl+C`: salir.

Con `--exit-on-open`, el programa se cierra después de abrir libros.

Las rutas se imprimen en stdout, una por línea, mientras la interfaz sigue en la terminal, así que puedes usarlas en scripts:

```bash
zathura "$(calibre-tui)"                    # abrir un libro
calibre-tui | xargs -d '\n' -r cp -t ~/usb  # copiar los libros seleccionados (GNU xargs)
```

## Configuración

En el primer arranque se crean cuatro archivos con valores predeterminados comentados:

- `config.toml`: ruta de la biblioteca, programas por formato, transliteraciones de búsqueda
- `layout.toml`: columnas, su orden y ancho, y qué campos se buscan
- `keymap.toml`: atajos de teclado
- `theme.toml`: colores

Están en `~/.config/calibre-tui/` en Linux, `~/Library/Application Support/calibre-tui/` en macOS y `%APPDATA%\calibre-tui\` en Windows. Cuando una actualización añade ajustes, se completan con sus valores predeterminados. Un archivo que ya no se puede leer se guarda como `<nombre>.bak-<marca de tiempo>` y se reemplaza por los valores predeterminados.

## Documentación

La documentación completa está en inglés:

- [Inicio rápido](quick-start.md)
- [Controles](controls.md) y [comandos](commands.md)
- [Búsqueda](search.md)
- [Configuración](configuration.md), [diseño de columnas](layout.md), [atajos](keymap.md), [tema](theme.md)
- [Solución de problemas](troubleshooting.md)
- [Arquitectura](architecture.md), para colaboradores
