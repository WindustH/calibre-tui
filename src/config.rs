use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  fs,
  path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
  pub library_path: PathBuf,
  pub filter: FilterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FilterConfig {
  pub translators: Vec<FilterTranslator>,
  pub pinyin_fuzzy: bool,
  pub pinyin_fuzzy_groups: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FilterTranslator {
  Pinyin,
  Romaji,
  GermanLatin,
  FrenchLatin,
  SpanishLatin,
  RussianLatin,
}

impl Default for Config {
  fn default() -> Self {
    Self {
      library_path: find_calibre_library().unwrap_or_default(),
      filter: FilterConfig::default(),
    }
  }
}

impl Default for FilterConfig {
  fn default() -> Self {
    Self {
      translators: vec![FilterTranslator::Pinyin],
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

pub fn load_config() -> Result<Config> {
  let config_dir = dirs::config_dir()
    .context("could not get config directory")?
    .join("calibre-tui");
  fs::create_dir_all(&config_dir)
    .with_context(|| format!("failed to create config directory at {:?}", config_dir))?;

  let config_path = config_dir.join("config.toml");
  if !config_path.exists() {
    let default_config = toml::to_string_pretty(&Config::default())?;
    fs::write(&config_path, default_config)
      .with_context(|| format!("failed to write config file to {:?}", config_path))?;
  }

  let content = fs::read_to_string(&config_path)
    .with_context(|| format!("failed to read config file: {:?}", config_path))?;
  let mut config: Config = toml::from_str(&content)
    .with_context(|| format!("failed to parse config file '{}'", config_path.display()))?;

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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_translator_names() {
    let config: Config = toml::from_str(
      r#"
library_path = "/tmp"

[filter]
translators = ["pinyin", "romaji", "german-latin", "french-latin", "spanish-latin", "russian-latin"]
"#,
    )
    .unwrap();

    assert_eq!(
      config.filter.translators,
      vec![
        FilterTranslator::Pinyin,
        FilterTranslator::Romaji,
        FilterTranslator::GermanLatin,
        FilterTranslator::FrenchLatin,
        FilterTranslator::SpanishLatin,
        FilterTranslator::RussianLatin,
      ]
    );
  }
}
