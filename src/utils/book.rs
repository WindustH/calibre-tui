use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Book {
  pub path: PathBuf,
  pub title: String,
  pub authors: Vec<String>,
  pub series: String,
  pub formats: Vec<String>,
  pub tags: Vec<String>,
}
