//! Calibre library data: book records, metadata fields, and library detection.

mod db;

pub use db::load_books;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::{env, fs};

#[derive(Debug, Clone, Default)]
pub struct Book {
  pub title: String,
  pub authors: Vec<String>,
  pub series: String,
  pub formats: Vec<String>,
  pub tags: Vec<String>,
  /// File opened for this book, or `None` for metadata-only entries.
  pub path: Option<PathBuf>,
}

/// A book metadata field shown in the table, searched, and sorted on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookField {
  Title,
  Authors,
  Series,
  Formats,
  Tags,
}

impl Book {
  /// Display text of a field. Search highlights index into this exact text,
  /// so every consumer must render fields through this function.
  pub fn field_text(&self, field: BookField) -> Cow<'_, str> {
    match field {
      BookField::Title => Cow::Borrowed(&self.title),
      BookField::Authors => join(&self.authors, " & "),
      BookField::Series => Cow::Borrowed(&self.series),
      BookField::Formats => join(&self.formats, ", "),
      BookField::Tags => join(&self.tags, ", "),
    }
  }
}

fn join<'a>(items: &'a [String], separator: &str) -> Cow<'a, str> {
  match items {
    [] => Cow::Borrowed(""),
    [item] => Cow::Borrowed(item),
    _ => Cow::Owned(items.join(separator)),
  }
}

impl BookField {
  pub fn parse(input: &str) -> Option<Self> {
    match input.to_ascii_lowercase().as_str() {
      "title" | "name" => Some(Self::Title),
      "author" | "authors" => Some(Self::Authors),
      "series" => Some(Self::Series),
      "format" | "formats" => Some(Self::Formats),
      "tag" | "tags" => Some(Self::Tags),
      _ => None,
    }
  }

  pub fn name(self) -> &'static str {
    match self {
      Self::Title => "title",
      Self::Authors => "authors",
      Self::Series => "series",
      Self::Formats => "formats",
      Self::Tags => "tags",
    }
  }
}

pub fn is_library(path: &Path) -> bool {
  path.join("metadata.db").is_file()
}

/// Find a Calibre library, preferring the one Calibre itself last opened.
pub fn find_library() -> Option<PathBuf> {
  candidate_paths().into_iter().find(|path| is_library(path))
}

fn candidate_paths() -> Vec<PathBuf> {
  let mut paths = Vec::new();
  paths.extend(calibre_configured_library());
  if let Some(home_dir) = dirs::home_dir() {
    paths.push(home_dir.join("Calibre Library"));
    paths.push(home_dir.join("Calibre-Bibliothek"));
  }
  if let Some(docs_dir) = dirs::document_dir() {
    paths.push(docs_dir.join("Calibre Library"));
  }
  paths
}

/// `library_path` from Calibre's `global.py.json`.
fn calibre_configured_library() -> Option<PathBuf> {
  let content = fs::read_to_string(calibre_config_dir()?.join("global.py.json")).ok()?;
  let json = serde_json::from_str::<Value>(&content).ok()?;
  json
    .get("library_path")
    .and_then(Value::as_str)
    .map(PathBuf::from)
}

/// Calibre's own config directory: `CALIBRE_CONFIG_DIRECTORY`, otherwise
/// `~/.config/calibre` (Linux), `~/Library/Preferences/calibre` (macOS),
/// or `%APPDATA%\calibre` (Windows).
fn calibre_config_dir() -> Option<PathBuf> {
  if let Some(dir) = env::var_os("CALIBRE_CONFIG_DIRECTORY") {
    return Some(PathBuf::from(dir));
  }
  dirs::preference_dir().map(|dir| dir.join("calibre"))
}
