use anyhow::{Result, anyhow};
use std::path::PathBuf;

// default app config
impl Default for super::App {
    fn default() -> Self {
        let path = PathBuf::from("");
        Self { library_path: path }
    }
}

/// check database in the library directory and convert str into pathbuf
pub fn validate_library_path(config_path_str: &String) -> Result<PathBuf> {
    if !config_path_str.is_empty() {
        let path = PathBuf::from(config_path_str);
        if path.join("metadata.db").exists() {
            return Ok(path);
        }
    }
    // check home dir for calibre library
    if let Some(home_dir) = dirs::home_dir() {
        let paths_to_check = [
            home_dir.join("Calibre Library"),
            home_dir.join("Calibre-Bibliothek"),
        ];
        for path in paths_to_check.iter() {
            if path.join("metadata.db").exists() {
                return Ok(path.clone());
            }
        }
    }
    // check document dir for calibre library
    if let Some(docs_dir) = dirs::document_dir() {
        let docs_path = docs_dir.join("Calibre Library");
        if docs_path.join("metadata.db").exists() {
            return Ok(docs_path);
        }
    }
    Err(anyhow!("database not found"))
}
