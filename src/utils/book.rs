use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Book {
  pub path: PathBuf,
  pub title: String,
  pub authors: Vec<String>,
  pub series: String,
  pub tags: Vec<String>,
}
