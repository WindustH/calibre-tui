use std::path::PathBuf;
use anyhow::Result;
use crate::utils::db::DbError;


// default app config
impl Default for super::App {
    fn default() -> Self {
        match validate_library_path(&"".to_string()){
            Ok(library_path)=>Self { library_path},
            Err(e)=>panic!("Can't find a valid calibre library path! Error: {:?}",e)
        }

    }
}

/// check database in the library directory and convert str into pathbuf
pub fn validate_library_path(config_path_str : &String) -> Result<PathBuf> {
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

    Err(DbError::DbNotFound.into())
}