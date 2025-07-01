use super::super::{App, app::validate_library_path};
use anyhow::Result;
use serde::Deserialize;

// validate the calibre library path
#[derive(Debug, Deserialize)]
pub struct Raw {
    pub library_path: String,
}

impl TryFrom<Raw> for App {
    type Error = anyhow::Error;
    fn try_from(raw: Raw) -> Result<Self, Self::Error> {
        let library_path = validate_library_path(&raw.library_path)?;
        Ok(App { library_path })
    }
}
