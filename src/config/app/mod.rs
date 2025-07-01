use anyhow::{anyhow, Result};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

// default app config
impl Default for super::App {
    fn default() -> Self {
        let path = PathBuf::from("");
        Self {
            library_path: path,
        }
    }
}

/// check database in the library directory and convert str into pathbuf
pub fn validate_library_path(config_path_str: &String) -> Result<PathBuf> {
    // check user provided path
    if !config_path_str.is_empty() {
        let path = PathBuf::from(config_path_str);
        if path.join("metadata.db").exists() {
            return Ok(path);
        }
    }

    // check calibre config
    if let Some(home_dir) = dirs::home_dir() {
        let calibre_config_path = home_dir.join(".config/calibre/global.py.json");
        if calibre_config_path.exists() {
            let content = fs::read_to_string(calibre_config_path)?;
            let json: Value = serde_json::from_str(&content)?;
            if let Some(library_path_val) = json.get("library_path") {
                if let Some(library_path_str) = library_path_val.as_str() {
                    let path = PathBuf::from(library_path_str);
                    if path.join("metadata.db").exists() {
                        return Ok(path);
                    }
                }
            }
        }
    }

    // fallback to check common path
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
    if let Some(docs_dir) = dirs::document_dir() {
        let docs_path = docs_dir.join("Calibre Library");
        if docs_path.join("metadata.db").exists() {
            return Ok(docs_path);
        }
    }

    Err(anyhow!("database not found"))
}