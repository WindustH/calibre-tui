use std::{fs, path::PathBuf};

use super::super::App;
use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;

// validate the calibre library path
#[derive(Debug, Deserialize)]
pub struct Raw {
    pub library_path: String,
}

impl TryFrom<Raw> for App {
    type Error = anyhow::Error;
    fn try_from(raw: Raw) -> Result<Self, Self::Error> {
        let library_path = (|| {
            if is_valid_library_path(&PathBuf::from(&raw.library_path)) {
                return Ok(PathBuf::from(&raw.library_path));
            } else {
                // check possible library paths
                for path in possible_library_paths() {
                    if is_valid_library_path(&path) {
                        return Ok(PathBuf::from(path));
                    }
                }
            }
            Err(anyhow::anyhow!("no valid library path found"))
        })()
        .with_context(|| format!("invalid library path: {}", raw.library_path))?;

        Ok(App {
            library_path,
            default_instance: "filter-and-open".to_string(),
        })
    }
}

/// check if the library path is valid
pub fn is_valid_library_path(path: &PathBuf) -> bool {
    path.join("metadata.db").exists()
}

/// return a list of possible calibre library paths to check
pub fn possible_library_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home_dir) = dirs::home_dir() {
        // common calibre library locations
        paths.push(home_dir.join("Calibre Library"));
        paths.push(home_dir.join("Calibre-Bibliothek"));

        // calibre config file
        let calibre_config_path = home_dir.join(".config/calibre/global.py.json");
        if calibre_config_path.exists() {
            if let Ok(content) = fs::read_to_string(&calibre_config_path) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    if let Some(library_path_val) = json.get("library_path") {
                        if let Some(library_path_str) = library_path_val.as_str() {
                            paths.push(PathBuf::from(library_path_str));
                        }
                    }
                }
            }
        }
    }

    if let Some(docs_dir) = dirs::document_dir() {
        paths.push(docs_dir.join("Calibre Library"));
    }

    paths
}
