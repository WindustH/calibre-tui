use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
pub mod app;
pub mod widget;
pub mod i18n;
pub mod ui;
pub mod pipeline;

pub mod validate;
mod default;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pipeline {
    pub instances:Vec<pipeline::Instance>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "validate::app::Raw")]
pub struct App {
    pub library_path: PathBuf,
    pub default_instance: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct I18n {
    pub filter: i18n::Filter,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]

pub struct Ui {
    #[serde(default)]
    pub filter: ui::Filter
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    #[serde(default)]
    pub app: App,
    #[serde(default)]
    pub i18n: I18n,
    #[serde(default)]
    pub ui: Ui,
    #[serde(default)]
    pub pipeline: Pipeline
}

/// load and parse config file
/// if config file or directory doesn't exist, use `Default` impl to create a config file
pub fn load_config() -> Result<Config> {
    let config_dir = dirs::config_dir().context("could not get config directory")?;
    let app_config_dir = config_dir.join("calibre-tui");

    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir).with_context(|| {
            format!("failed to create config directory at {:?}", app_config_dir)
        })?;
    }

    let config_file_path = app_config_dir.join("config.toml");

    // get default config
    let config = Config::default();

    // create default config
    if !config_file_path.exists() {
        let default_toml_content = toml::to_string_pretty(&config)?;
        fs::write(&config_file_path, default_toml_content).with_context(|| {
            format!(
                "failed to write default config file to {:?}",
                config_file_path
            )
        })?;
    }

    // read config
    let content = fs::read_to_string(&config_file_path)
        .with_context(|| format!("failed to read config file: {:?}", config_file_path))?;

    let final_config: Config = toml::from_str(&content)
        .with_context(|| {
            format!(
                "fail to parse config file '{}'",
                config_file_path.display()
            )
        })?;

    Ok(final_config)
}
