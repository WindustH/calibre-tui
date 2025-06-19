use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::Deserialize;
use std::{fs, path::PathBuf, str::FromStr};

pub fn parse_color(s: &str) -> Color {
    if let Ok(color) = Color::from_str(&s.to_lowercase()) {
        return color;
    }
    if let Some(hex_str) = s.strip_prefix('#') {
        if let Ok(rgb) = u32::from_str_radix(hex_str, 16) {
            let r = ((rgb >> 16) & 0xFF) as u8;
            let g = ((rgb >> 8) & 0xFF) as u8;
            let b = (rgb & 0xFF) as u8;
            return Color::Rgb(r, g, b);
        }
    }
    if let Some(rgb_str) = s.strip_prefix("rgb(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<Result<u8, _>> = rgb_str.split(',').map(|p| p.trim().parse()).collect();
        if parts.len() == 3 {
            if let (Ok(r), Ok(g), Ok(b)) = (&parts[0], &parts[1], &parts[2]) {
                return Color::Rgb(*r, *g, *b);
            }
        }
    }
    Color::Reset
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ColorsConfig {
    #[serde(rename = "search-box-border-fg")]
    pub search_box_border_fg: String,
    #[serde(rename = "table-border-fg")]
    pub table_border_fg: String,
    #[serde(rename = "search-box-title")]
    pub search_box_title: String,
    #[serde(rename = "search-box-text")]
    pub search_box_text: String,
    #[serde(rename = "table-title-fg")]
    pub table_title_fg: String,
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            search_box_border_fg: "Blue".to_string(),
            table_border_fg: "Blue".to_string(),
            search_box_title: "Blue".to_string(),
            search_box_text: "White".to_string(),
            table_title_fg: "Blue".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct ColumnConfig {
    pub name: String,
    #[serde(rename = "width-ratio")]
    pub width_ratio: u16,
    pub fg: String,
    pub bg: String,
    #[serde(rename = "selected-fg")]
    pub selected_fg: String,
    #[serde(rename = "selected-bg")]
    pub selected_bg: String,
    #[serde(rename = "highlighted-match-fg")]
    pub highlighted_match_fg: String,
    #[serde(rename = "header-fg")]
    pub header_fg: String,
}

impl Default for ColumnConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            width_ratio: 25,
            fg: "White".to_string(),
            bg: "Reset".to_string(),
            selected_fg: "White".to_string(),
            selected_bg: "Blue".to_string(),
            highlighted_match_fg: "Red".to_string(),
            header_fg: "Blue".to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    #[serde(rename = "library-path")]
    pub library_path: Option<String>,
    #[serde(rename = "pinyin-search-enabled")]
    pub pinyin_search_enabled: bool,
    #[serde(rename = "fuzzy-pinyin")]
    pub fuzzy_pinyin: Vec<Vec<String>>,
    pub columns: Vec<ColumnConfig>,
    pub colors: ColorsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            library_path: None,
            pinyin_search_enabled: false,
            fuzzy_pinyin: vec![
                vec!["on".to_string(), "ong".to_string()],
                vec!["an".to_string(), "ang".to_string()],
                vec!["en".to_string(), "eng".to_string()],
                vec!["in".to_string(), "ing".to_string()],
            ],
            columns: vec![
                ColumnConfig {
                    name: "title".to_string(),
                    width_ratio: 40,
                    fg: "White".to_string(),
                    selected_bg: "Blue".to_string(),
                    ..Default::default()
                },
                ColumnConfig {
                    name: "author".to_string(),
                    width_ratio: 20,
                    fg: "Cyan".to_string(),
                    ..Default::default()
                },
                ColumnConfig {
                    name: "series".to_string(),
                    width_ratio: 20,
                    fg: "White".to_string(),
                    ..Default::default()
                },
                ColumnConfig {
                    name: "tags".to_string(),
                    width_ratio: 20,
                    fg: "Cyan".to_string(),
                    ..Default::default()
                },
            ],
            colors: ColorsConfig::default(),
        }
    }
}

const DEFAULT_CONFIG: &str = "\
# Calibre 书库的路径。\n\
library-path = \"\"\n\
\n\
# --- 搜索功能配置 ---\n\
# 是否启用拼音搜索功能 (默认为关闭)。\n\
pinyin-search-enabled = false\n\
\n\
# 模糊音匹配规则组 (仅在 pinyin-search-enabled = true 时生效)。\n\
fuzzy-pinyin = [\n\
    [\"on\", \"ong\"],\n\
    [\"an\", \"ang\"],\n\
    [\"en\", \"eng\"],\n\
    [\"in\", \"ing\"],\n\
]\n\
\n\
# --- UI 列布局和颜色 ---\n\
[[columns]]\n\
name = \"title\"\n\
width-ratio = 40\n\
fg = \"White\"\n\
bg = \"Reset\"\n\
selected-fg = \"White\"\n\
selected-bg = \"Blue\"\n\
header-fg = \"Blue\"\n\
highlighted-match-fg = \"Red\"\n\
\n\
[[columns]]\n\
name = \"author\"\n\
width-ratio = 20\n\
fg = \"Cyan\"\n\
bg = \"Reset\"\n\
selected-fg = \"White\"\n\
selected-bg = \"Blue\"\n\
header-fg = \"Blue\"\n\
highlighted-match-fg = \"Red\"\n\
\n\
[[columns]]\n\
name = \"series\"\n\
width-ratio = 20\n\
fg = \"White\"\n\
bg = \"Reset\"\n\
selected-fg = \"White\"\n\
selected-bg = \"Blue\"\n\
header-fg = \"Blue\"\n\
highlighted-match-fg = \"Red\"\n\
\n\
[[columns]]\n\
name = \"tags\"\n\
width-ratio = 20\n\
fg = \"Cyan\"\n\
bg = \"Reset\"\n\
selected-fg = \"White\"\n\
selected-bg = \"Blue\"\n\
header-fg = \"Blue\"\n\
highlighted-match-fg = \"Red\"\n\
\n\
# --- 全局颜色配置 ---\n\
[colors]\n\
search-box-border-fg = \"Blue\"\n\
table-border-fg = \"Blue\"\n\
search-box-title = \"Blue\"\n\
search-box-text = \"White\"\n\
table-title-fg = \"Blue\"\n\
";

pub fn load_config() -> Result<Config> {
    let config_dir = dirs::config_dir().context("Could not get config directory")?;
    let app_config_dir = config_dir.join("calibre-tui");

    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)
            .with_context(|| format!("Failed to create config directory at {:?}", app_config_dir))?;
    }

    let config_file_path = app_config_dir.join("config.toml");

    if !config_file_path.exists() {
        fs::write(&config_file_path, DEFAULT_CONFIG)
            .with_context(|| format!("Failed to write default config file to {:?}", config_file_path))?;
    }

    let content = fs::read_to_string(&config_file_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_file_path))?;
    let config: Config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file '{}'", config_file_path.display()))?;

    Ok(config)
}

pub fn find_calibre_library_path(config: &Config) -> Option<PathBuf> {
    if let Some(config_path_str) = &config.library_path {
        if !config_path_str.is_empty() {
            let path = PathBuf::from(config_path_str);
            if path.join("metadata.db").exists() {
                return Some(path);
            }
        }
    }

    if let Some(home_dir) = dirs::home_dir() {
        let paths_to_check = [
            home_dir.join("Calibre Library"),
            home_dir.join("Calibre-Bibliothek"),
        ];
        for path in paths_to_check.iter() {
            if path.join("metadata.db").exists() {
                return Some(path.clone());
            }
        }
    }

    if let Some(docs_dir) = dirs::document_dir() {
        let docs_path = docs_dir.join("Calibre Library");
        if docs_path.join("metadata.db").exists() {
            return Some(docs_path);
        }
    }

    None
}
