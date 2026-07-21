use crate::config_file::{
  CommentedToml, TomlComment, app_config_dir, load_toml_or_reset_with, serialize_with_comments,
};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct Layout {
  columns: Vec<LayoutColumn>,
}

#[derive(Debug, Clone)]
pub struct LayoutColumn {
  pub field: BookField,
  pub label: String,
  pub visible: bool,
  pub search: bool,
  pub width: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookField {
  Title,
  Authors,
  Series,
  Formats,
  Tags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(default)]
struct LayoutConfig {
  columns: Vec<LayoutColumnConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutColumnConfig {
  field: BookField,
  #[serde(default)]
  label: Option<String>,
  #[serde(default = "default_true")]
  visible: bool,
  #[serde(default = "default_true")]
  search: bool,
  #[serde(default = "default_width")]
  width: u16,
}

impl Layout {
  pub fn visible_columns(&self) -> impl Iterator<Item = &LayoutColumn> {
    self.columns.iter().filter(|column| column.visible)
  }

  pub fn search_fields(&self) -> impl Iterator<Item = BookField> + '_ {
    self
      .columns
      .iter()
      .filter(|column| column.search)
      .map(|column| column.field)
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

  fn default_label(self) -> &'static str {
    self.name()
  }
}

impl Default for LayoutConfig {
  fn default() -> Self {
    Self {
      columns: vec![
        LayoutColumnConfig::new(BookField::Title, 35),
        LayoutColumnConfig::new(BookField::Authors, 20),
        LayoutColumnConfig::new(BookField::Series, 18),
        LayoutColumnConfig::new(BookField::Formats, 12),
        LayoutColumnConfig::new(BookField::Tags, 15),
      ],
    }
  }
}

impl LayoutColumnConfig {
  fn new(field: BookField, width: u16) -> Self {
    Self {
      field,
      label: None,
      visible: true,
      search: true,
      width,
    }
  }
}

impl CommentedToml for LayoutConfig {
  fn comments() -> &'static [TomlComment] {
    &[
      TomlComment {
        path: "",
        lines: &[
          "Table column layout.",
          "The order of [[columns]] entries controls table order and search match priority.",
          "Supported fields: title, authors, series, formats, tags.",
        ],
      },
      TomlComment {
        path: "columns",
        lines: &["One table/search field. Repeated column fields are documented only once."],
      },
      TomlComment {
        path: "columns.field",
        lines: &["Book metadata field represented by this column."],
      },
      TomlComment {
        path: "columns.label",
        lines: &["Column title displayed in the table header."],
      },
      TomlComment {
        path: "columns.visible",
        lines: &["Show this field as a table column."],
      },
      TomlComment {
        path: "columns.search",
        lines: &["Include this field in search matching and highlighting."],
      },
      TomlComment {
        path: "columns.width",
        lines: &["Relative table width. Values are proportions and do not need to add up to 100."],
      },
    ]
  }

  fn to_commented_toml(&self) -> Result<String> {
    let mut normalized = self.clone();
    for column in &mut normalized.columns {
      if column.label.is_none() {
        column.label = Some(column.field.default_label().to_string());
      }
    }
    serialize_with_comments(&normalized, Self::comments())
  }
}

impl LayoutConfig {
  fn compile(self) -> Result<Layout> {
    if self.columns.is_empty() {
      bail!("layout must define at least one column");
    }

    let mut seen = BTreeSet::new();
    let mut columns = Vec::new();
    for column in self.columns {
      if !seen.insert(column.field) {
        bail!("duplicate layout column '{:?}'", column.field);
      }

      if column.visible && column.width == 0 {
        bail!(
          "visible layout column '{:?}' must have width > 0",
          column.field
        );
      }

      columns.push(LayoutColumn {
        field: column.field,
        label: column
          .label
          .unwrap_or_else(|| column.field.default_label().to_string()),
        visible: column.visible,
        search: column.search,
        width: column.width,
      });
    }

    if !columns.iter().any(|column| column.visible) {
      bail!("layout must have at least one visible column");
    }

    Ok(Layout { columns })
  }
}

pub fn load_layout() -> Result<Layout> {
  let config_dir = app_config_dir()?;
  let layout_path = config_dir.join("layout.toml");
  load_toml_or_reset_with(
    &layout_path,
    LayoutConfig::default(),
    "layout",
    LayoutConfig::compile,
  )
  .with_context(|| format!("failed to load layout file '{}'", layout_path.display()))
}

fn default_true() -> bool {
  true
}

fn default_width() -> u16 {
  1
}
