use crate::config_file::{CommentedToml, TomlComment, app_config_dir, load_toml_or_reset};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct Config {
  pub library_path: PathBuf,
  pub open: OpenConfig,
  pub filter: FilterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct OpenConfig {
  pub commands: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
pub struct FilterConfig {
  pub translators: Vec<FilterTranslator>,
  pub pinyin_fuzzy: bool,
  pub pinyin_fuzzy_groups: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilterTranslator {
  #[serde(rename = "pinyin")]
  ChinesePinyin,
  #[serde(rename = "romaji")]
  JapaneseRomaji,
  GermanLatin,
  FrenchLatin,
  SpanishLatin,
  RussianLatin,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      library_path: find_calibre_library().unwrap_or_default(),
      open: OpenConfig::default(),
      filter: FilterConfig::default(),
    }
  }
}

impl Default for OpenConfig {
  fn default() -> Self {
    Self {
      commands: BTreeMap::new(),
    }
  }
}

impl Default for FilterConfig {
  fn default() -> Self {
    Self {
      translators: vec![FilterTranslator::ChinesePinyin],
      pinyin_fuzzy: true,
      pinyin_fuzzy_groups: vec![
        vec!["on".to_string(), "ong".to_string()],
        vec!["an".to_string(), "ang".to_string()],
        vec!["en".to_string(), "eng".to_string()],
        vec!["in".to_string(), "ing".to_string()],
      ],
    }
  }
}

impl CommentedToml for Config {
  fn comments() -> &'static [TomlComment] {
    &[
      TomlComment {
        path: "",
        lines: &[
          "Main calibre-tui configuration.",
          "Missing fields are filled with defaults and this file is rewritten with comments.",
        ],
      },
      TomlComment {
        path: "library_path",
        lines: &[
          "Path to the Calibre library directory.",
          "Leave empty to auto-detect common locations.",
        ],
      },
      TomlComment {
        path: "open",
        lines: &["File opening options."],
      },
      TomlComment {
        path: "open.commands",
        lines: &[
          "Format-specific opener commands. Formats are matched by file extension.",
          "Leave this table empty to use the system opener for every format.",
          "Keys are matched case-insensitively; use names like pdf, epub, mobi, cbz.",
          "Use {path} in any argument to choose where the file path is inserted.",
          "If {path} is omitted, the path is appended as the last argument.",
          "Example: pdf = [\"zathura\", \"{path}\"]",
        ],
      },
      TomlComment {
        path: "filter",
        lines: &["Search indexing and text normalization options."],
      },
      TomlComment {
        path: "filter.translators",
        lines: &[
          "Search translators to enable.",
          "Supported values: pinyin, romaji, german-latin, french-latin, spanish-latin, russian-latin.",
        ],
      },
      TomlComment {
        path: "filter.pinyin_fuzzy",
        lines: &["Enable fuzzy matching for Chinese pinyin fragments."],
      },
      TomlComment {
        path: "filter.pinyin_fuzzy_groups",
        lines: &[
          "Equivalent pinyin fragments.",
          "The first item in each group is treated as canonical.",
        ],
      },
    ]
  }
}

pub fn load_config() -> Result<Config> {
  let config_dir = app_config_dir()?;
  let config_path = config_dir.join("config.toml");
  let mut config: Config = load_toml_or_reset(&config_path, Config::default(), "main")?;

  if config.library_path.as_os_str().is_empty() {
    config.library_path = find_calibre_library()
      .context("library_path is empty and no Calibre library was found in common locations")?;
  }

  if !is_calibre_library(&config.library_path) {
    bail!(
      "invalid Calibre library path '{}': metadata.db was not found",
      config.library_path.display()
    );
  }

  Ok(config)
}

fn is_calibre_library(path: &Path) -> bool {
  path.join("metadata.db").exists()
}

fn find_calibre_library() -> Option<PathBuf> {
  possible_library_paths()
    .into_iter()
    .find(|path| is_calibre_library(path.as_path()))
}

fn possible_library_paths() -> Vec<PathBuf> {
  let mut paths = Vec::new();

  if let Some(home_dir) = dirs::home_dir() {
    paths.push(home_dir.join("Calibre Library"));
    paths.push(home_dir.join("Calibre-Bibliothek"));

    let calibre_config_path = home_dir.join(".config/calibre/global.py.json");
    if let Ok(content) = fs::read_to_string(calibre_config_path)
      && let Ok(json) = serde_json::from_str::<Value>(&content)
      && let Some(library_path) = json.get("library_path").and_then(Value::as_str)
    {
      paths.push(PathBuf::from(library_path));
    }
  }

  if let Some(docs_dir) = dirs::document_dir() {
    paths.push(docs_dir.join("Calibre Library"));
  }

  paths
}
