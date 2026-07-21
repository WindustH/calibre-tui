use crate::config_file::{CommentedToml, TomlComment, app_config_dir, load_toml_or_reset};
use anyhow::Result;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Theme {
  pub foreground: String,
  pub background: String,
  pub accent: String,
  pub muted: String,
  pub search: SearchTheme,
  pub command: CommandTheme,
  pub table: TableTheme,
  pub row: RowTheme,
  pub highlight: HighlightTheme,
  pub footer: FooterTheme,
  pub completion: CompletionTheme,
  pub help: HelpTheme,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct SearchTheme {
  pub border: String,
  pub title: String,
  pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CommandTheme {
  pub border: String,
  pub title: String,
  pub text: String,
  pub prefix: String,
  pub suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct TableTheme {
  pub border: String,
  pub title: String,
  pub header: String,
  pub title_field: String,
  pub authors_field: String,
  pub series_field: String,
  pub formats_field: String,
  pub tags_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct RowTheme {
  pub hover_foreground: String,
  pub hover_background: String,
  pub selected_foreground: String,
  pub selected_background: String,
  pub selected_hover_foreground: String,
  pub selected_hover_background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct HighlightTheme {
  pub normal: String,
  pub hover: String,
  pub selected: String,
  pub selected_hover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FooterTheme {
  pub message: String,
  pub which_key_background: String,
  pub which_key_foreground: String,
  pub which_key_key: String,
  pub which_key_separator: String,
  pub which_key_description: String,
  pub which_key_separator_text: String,
  pub which_key_columns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct CompletionTheme {
  pub foreground: String,
  pub background: String,
  pub selected_foreground: String,
  pub selected_background: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct HelpTheme {
  pub background: String,
  pub border: String,
  pub key: String,
  pub description: String,
  pub muted: String,
}

impl Default for Theme {
  fn default() -> Self {
    Self {
      foreground: "white".to_string(),
      background: "reset".to_string(),
      accent: "blue".to_string(),
      muted: "dark_gray".to_string(),
      search: SearchTheme::default(),
      command: CommandTheme::default(),
      table: TableTheme::default(),
      row: RowTheme::default(),
      highlight: HighlightTheme::default(),
      footer: FooterTheme::default(),
      completion: CompletionTheme::default(),
      help: HelpTheme::default(),
    }
  }
}

impl Default for SearchTheme {
  fn default() -> Self {
    Self {
      border: "blue".to_string(),
      title: "blue".to_string(),
      text: "white".to_string(),
    }
  }
}

impl Default for CommandTheme {
  fn default() -> Self {
    Self {
      border: "blue".to_string(),
      title: "blue".to_string(),
      text: "white".to_string(),
      prefix: "blue".to_string(),
      suggestion: "dark_gray".to_string(),
    }
  }
}

impl Default for TableTheme {
  fn default() -> Self {
    Self {
      border: "blue".to_string(),
      title: "blue".to_string(),
      header: "blue".to_string(),
      title_field: "white".to_string(),
      authors_field: "cyan".to_string(),
      series_field: "white".to_string(),
      formats_field: "magenta".to_string(),
      tags_field: "cyan".to_string(),
    }
  }
}

impl Default for RowTheme {
  fn default() -> Self {
    Self {
      hover_foreground: "black".to_string(),
      hover_background: "blue".to_string(),
      selected_foreground: "black".to_string(),
      selected_background: "green".to_string(),
      selected_hover_foreground: "black".to_string(),
      selected_hover_background: "yellow".to_string(),
    }
  }
}

impl Default for HighlightTheme {
  fn default() -> Self {
    Self {
      normal: "red".to_string(),
      hover: "yellow".to_string(),
      selected: "yellow".to_string(),
      selected_hover: "red".to_string(),
    }
  }
}

impl Default for FooterTheme {
  fn default() -> Self {
    Self {
      message: "dark_gray".to_string(),
      which_key_background: "reset".to_string(),
      which_key_foreground: "white".to_string(),
      which_key_key: "yellow".to_string(),
      which_key_separator: "dark_gray".to_string(),
      which_key_description: "white".to_string(),
      which_key_separator_text: "  ".to_string(),
      which_key_columns: 3,
    }
  }
}

impl Default for CompletionTheme {
  fn default() -> Self {
    Self {
      foreground: "white".to_string(),
      background: "reset".to_string(),
      selected_foreground: "black".to_string(),
      selected_background: "blue".to_string(),
    }
  }
}

impl Default for HelpTheme {
  fn default() -> Self {
    Self {
      background: "reset".to_string(),
      border: "blue".to_string(),
      key: "yellow".to_string(),
      description: "white".to_string(),
      muted: "dark_gray".to_string(),
    }
  }
}

impl Theme {
  pub fn color(&self, value: &str) -> Color {
    parse_color(value)
  }
}

impl CommentedToml for Theme {
  fn comments() -> &'static [TomlComment] {
    &[
      TomlComment {
        path: "",
        lines: &[
          "Theme colors accept named ratatui colors, reset, ansi:<0-255>, or #rrggbb.",
          "Similar field names keep the same meaning across sections and are documented once.",
        ],
      },
      TomlComment {
        path: "foreground",
        lines: &["Base foreground used by general text."],
      },
      TomlComment {
        path: "background",
        lines: &["Base background used behind the application."],
      },
      TomlComment {
        path: "accent",
        lines: &["Accent color used for primary UI emphasis."],
      },
      TomlComment {
        path: "muted",
        lines: &["Muted color used for secondary text."],
      },
      TomlComment {
        path: "search",
        lines: &["Search input box colors."],
      },
      TomlComment {
        path: "search.border",
        lines: &["Border color. The same field name has the same role in other sections."],
      },
      TomlComment {
        path: "search.title",
        lines: &["Box title color. The same field name has the same role in other sections."],
      },
      TomlComment {
        path: "search.text",
        lines: &["Input text color."],
      },
      TomlComment {
        path: "command",
        lines: &["Command prompt colors."],
      },
      TomlComment {
        path: "command.prefix",
        lines: &["Command prompt prefix color."],
      },
      TomlComment {
        path: "command.suggestion",
        lines: &["Inline completion suggestion color."],
      },
      TomlComment {
        path: "table",
        lines: &["Book list table colors."],
      },
      TomlComment {
        path: "table.header",
        lines: &["Table header color."],
      },
      TomlComment {
        path: "table.title_field",
        lines: &["Per-field text colors for visible book columns."],
      },
      TomlComment {
        path: "row",
        lines: &["Row state colors for hover, selection, and selected-hover."],
      },
      TomlComment {
        path: "highlight",
        lines: &["Search match highlight colors for each row state."],
      },
      TomlComment {
        path: "footer",
        lines: &["Footer message and which-key hint colors."],
      },
      TomlComment {
        path: "footer.which_key_separator_text",
        lines: &["Text placed between a which-key key and its description."],
      },
      TomlComment {
        path: "footer.which_key_columns",
        lines: &[
          "Preferred which-key column count. It is reduced automatically on narrow terminals.",
        ],
      },
      TomlComment {
        path: "completion",
        lines: &["Command completion list colors."],
      },
      TomlComment {
        path: "completion.selected_background",
        lines: &[
          "Selected completion background color. Defaults to blue for consistency with hover.",
        ],
      },
      TomlComment {
        path: "help",
        lines: &["F1 help popup colors."],
      },
    ]
  }
}

pub fn load_theme() -> Result<Theme> {
  let config_dir = app_config_dir()?;
  let theme_path = config_dir.join("theme.toml");
  load_toml_or_reset(&theme_path, Theme::default(), "theme")
}

fn parse_color(value: &str) -> Color {
  let lower = value.trim().to_ascii_lowercase();
  match lower.as_str() {
    "reset" => Color::Reset,
    "black" => Color::Black,
    "red" => Color::Red,
    "green" => Color::Green,
    "yellow" => Color::Yellow,
    "blue" => Color::Blue,
    "magenta" => Color::Magenta,
    "cyan" => Color::Cyan,
    "gray" | "grey" => Color::Gray,
    "dark_gray" | "dark_grey" | "darkgray" | "darkgrey" => Color::DarkGray,
    "light_red" | "lightred" => Color::LightRed,
    "light_green" | "lightgreen" => Color::LightGreen,
    "light_yellow" | "lightyellow" => Color::LightYellow,
    "light_blue" | "lightblue" => Color::LightBlue,
    "light_magenta" | "lightmagenta" => Color::LightMagenta,
    "light_cyan" | "lightcyan" => Color::LightCyan,
    "white" => Color::White,
    _ => {
      if let Some(raw) = lower.strip_prefix("ansi:") {
        return raw
          .parse::<u8>()
          .map(Color::Indexed)
          .unwrap_or(Color::Reset);
      }
      if lower.len() == 7 && lower.starts_with('#') {
        let r = u8::from_str_radix(&lower[1..3], 16);
        let g = u8::from_str_radix(&lower[3..5], 16);
        let b = u8::from_str_radix(&lower[5..7], 16);
        if let (Ok(r), Ok(g), Ok(b)) = (r, g, b) {
          return Color::Rgb(r, g, b);
        }
      }
      Color::Reset
    }
  }
}
