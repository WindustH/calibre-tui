use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::{
  collections::{BTreeMap, BTreeSet},
  fs,
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

pub struct TomlComment {
  pub path: &'static str,
  pub lines: &'static [&'static str],
}

pub trait CommentedToml {
  fn comments() -> &'static [TomlComment];

  fn to_commented_toml(&self) -> Result<String>
  where
    Self: Serialize + Sized,
  {
    serialize_with_comments(self, Self::comments())
  }
}

pub fn serialize_with_comments<T>(value: &T, comments: &[TomlComment]) -> Result<String>
where
  T: Serialize,
{
  let document = compact_short_arrays(&toml::to_string_pretty(value)?);
  Ok(insert_comments(&document, comments))
}

pub fn app_config_dir() -> Result<PathBuf> {
  let config_dir = dirs::config_dir()
    .context("could not get config directory")?
    .join("calibre-tui");
  fs::create_dir_all(&config_dir)
    .with_context(|| format!("failed to create config directory at {:?}", config_dir))?;
  Ok(config_dir)
}

pub fn load_toml_or_reset<T>(path: &Path, default: T, label: &str) -> Result<T>
where
  T: Clone + Serialize + DeserializeOwned + CommentedToml,
{
  load_toml_or_reset_with(path, default, label, Ok)
}

pub fn load_toml_or_reset_with<T, U>(
  path: &Path,
  default: T,
  label: &str,
  compile: impl Fn(T) -> Result<U>,
) -> Result<U>
where
  T: Clone + Serialize + DeserializeOwned + CommentedToml,
{
  if !path.exists() {
    write_config(path, &default)?;
  }

  let content =
    fs::read_to_string(path).with_context(|| format!("failed to read config file: {:?}", path))?;
  match read_fill_and_compile(path, &content, default.clone(), &compile) {
    Ok((config, value, should_write)) => {
      if should_write {
        write_config(path, &config)?;
      }
      Ok(value)
    }
    Err(error) => {
      let backup_path = backup_file(path)?;
      write_config(path, &default)?;
      eprintln!(
        "Backed up incompatible {label} config to {} and generated a new default {}",
        backup_path.display(),
        path.display()
      );
      eprintln!("Previous {label} config error: {error:#}");
      compile(default)
    }
  }
}

fn read_fill_and_compile<T, U>(
  path: &Path,
  content: &str,
  default: T,
  compile: &impl Fn(T) -> Result<U>,
) -> Result<(T, U, bool)>
where
  T: Clone + Serialize + DeserializeOwned + CommentedToml,
{
  let mut value: toml::Value = toml::from_str(content)
    .with_context(|| format!("failed to parse config file '{}'", path.display()))?;
  let default_document = toml::to_string_pretty(&default)?;
  let default_value: toml::Value = toml::from_str(&default_document).with_context(|| {
    format!(
      "failed to parse built-in default config for '{}'",
      path.display()
    )
  })?;
  let mut added_paths = Vec::new();
  merge_missing_values(&mut value, &default_value, "", &mut added_paths);

  let config: T = value
    .try_into()
    .with_context(|| format!("failed to parse config file '{}'", path.display()))?;
  let compiled = compile(config.clone())?;
  let next_document = config.to_commented_toml()?;
  Ok((config, compiled, next_document != content))
}

fn write_config<T>(path: &Path, config: &T) -> Result<()>
where
  T: Serialize + CommentedToml,
{
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)
      .with_context(|| format!("failed to create config directory at {:?}", parent))?;
  }
  fs::write(path, config.to_commented_toml()?)
    .with_context(|| format!("failed to write config file to {:?}", path))?;
  Ok(())
}

fn merge_missing_values(
  target: &mut toml::Value,
  default: &toml::Value,
  path: &str,
  added_paths: &mut Vec<String>,
) {
  let (Some(target_table), Some(default_table)) = (target.as_table_mut(), default.as_table())
  else {
    return;
  };

  for (key, default_value) in default_table {
    let child_path = if path.is_empty() {
      key.clone()
    } else {
      format!("{path}.{key}")
    };
    if let Some(target_value) = target_table.get_mut(key) {
      merge_missing_values(target_value, default_value, &child_path, added_paths);
    } else {
      target_table.insert(key.clone(), default_value.clone());
      added_paths.push(child_path);
    }
  }
}

fn backup_file(path: &Path) -> Result<PathBuf> {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_secs())
    .unwrap_or(0);
  let file_name = path
    .file_name()
    .map(|name| name.to_string_lossy())
    .unwrap_or_else(|| "config.toml".into());

  for suffix in 0..1000 {
    let backup_name = if suffix == 0 {
      format!("{file_name}.bak-{timestamp}")
    } else {
      format!("{file_name}.bak-{timestamp}-{suffix}")
    };
    let backup_path = path.with_file_name(backup_name);
    if backup_path.exists() {
      continue;
    }
    fs::rename(path, &backup_path)
      .with_context(|| format!("failed to back up config file {:?}", path))?;
    return Ok(backup_path);
  }

  anyhow::bail!(
    "failed to choose a backup name for config file '{}'",
    path.display()
  )
}

fn insert_comments(document: &str, comments: &[TomlComment]) -> String {
  let comments = comments
    .iter()
    .map(|comment| (comment.path, comment.lines))
    .collect::<BTreeMap<_, _>>();
  let mut emitted = BTreeSet::<String>::new();
  let mut out = String::new();
  let mut current_table = String::new();

  emit_comment(&mut out, &comments, &mut emitted, "");

  for line in document.lines() {
    let trimmed = line.trim();
    if let Some(table) = table_path(trimmed) {
      current_table = table;
      emit_comment(&mut out, &comments, &mut emitted, &current_table);
    } else if let Some(key) = key_path(trimmed) {
      let path = if current_table.is_empty() {
        key
      } else {
        format!("{current_table}.{key}")
      };
      emit_comment(&mut out, &comments, &mut emitted, &path);
    }
    out.push_str(line);
    out.push('\n');
  }

  out
}

fn compact_short_arrays(document: &str) -> String {
  let mut current = document.to_string();

  loop {
    let next = compact_short_arrays_once(&current);
    if next == current {
      return current;
    }
    current = next;
  }
}

fn compact_short_arrays_once(document: &str) -> String {
  let lines = document.lines().collect::<Vec<_>>();
  let mut out = String::new();
  let mut index = 0;

  while index < lines.len() {
    if let Some((end, line)) = compact_array_at(&lines, index) {
      out.push_str(&line);
      out.push('\n');
      index = end + 1;
    } else {
      out.push_str(lines[index]);
      out.push('\n');
      index += 1;
    }
  }

  out
}

fn compact_array_at(lines: &[&str], start: usize) -> Option<(usize, String)> {
  const MAX_COMPACT_ARRAY_LINE_LEN: usize = 120;

  let line = lines[start];
  let trimmed = line.trim();
  if trimmed.is_empty() || trimmed.starts_with('#') {
    return None;
  }

  let open = line.find('[')?;
  if !line[open + 1..].trim().is_empty() {
    return None;
  }

  let prefix = &line[..open];
  let mut values = Vec::new();

  for (offset, child_line) in lines[start + 1..].iter().enumerate() {
    let child = child_line.trim();
    if child == "]" || child == "]," {
      if values.is_empty() {
        return None;
      }

      let trailing_comma = child.ends_with(',');
      let compact = format!(
        "{prefix}[{}]{}",
        values.join(", "),
        if trailing_comma { "," } else { "" }
      );
      if compact.len() <= MAX_COMPACT_ARRAY_LINE_LEN {
        return Some((start + offset + 1, compact));
      }
      return None;
    }

    if child.is_empty() || child.starts_with('#') || child.starts_with('[') || child.contains(']') {
      return None;
    }

    let value = child.strip_suffix(',')?;
    values.push(value.to_string());
  }

  None
}

fn emit_comment(
  out: &mut String,
  comments: &BTreeMap<&str, &[&str]>,
  emitted: &mut BTreeSet<String>,
  path: &str,
) {
  let Some(lines) = comments.get(path) else {
    return;
  };
  if !emitted.insert(path.to_string()) {
    return;
  }
  for line in *lines {
    if line.is_empty() {
      out.push_str("#\n");
    } else {
      out.push_str("# ");
      out.push_str(line);
      out.push('\n');
    }
  }
}

fn table_path(trimmed: &str) -> Option<String> {
  if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
    return Some(trimmed[2..trimmed.len() - 2].trim().to_string());
  }
  if trimmed.starts_with('[') && trimmed.ends_with(']') {
    return Some(trimmed[1..trimmed.len() - 1].trim().to_string());
  }
  None
}

fn key_path(trimmed: &str) -> Option<String> {
  if trimmed.starts_with('#') || trimmed.is_empty() {
    return None;
  }
  let (key, _) = trimmed.split_once('=')?;
  Some(key.trim().to_string())
}
