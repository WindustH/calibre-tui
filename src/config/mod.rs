use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
pub mod app;
pub mod command;
pub mod i18n;
pub mod ui;

pub mod validate;
mod default;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "validate::app::Raw")]
pub struct App {
    pub library_path: PathBuf,
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
}

/// load and parse config file
/// if config file or directory doesn't exist, use `Default` impl to create a config file
pub fn load_config() -> Result<Config> {
    let config_dir = dirs::config_dir().context("Could not get config directory")?;
    let app_config_dir = config_dir.join("calibre-tui");

    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir).with_context(|| {
            format!("Failed to create config directory at {:?}", app_config_dir)
        })?;
    }

    let config_file_path = app_config_dir.join("config.toml");

    // 首先获取默认配置
    let config = Config::default();

    // 如果配置文件不存在，则创建默认配置文件
    if !config_file_path.exists() {
        let default_toml_content = toml::to_string_pretty(&config)?; // 使用默认配置生成内容
        fs::write(&config_file_path, default_toml_content).with_context(|| {
            format!(
                "failed to write default config file to {:?}",
                config_file_path
            )
        })?;
    }

    // 读取用户配置文件的内容
    let content = fs::read_to_string(&config_file_path)
        .with_context(|| format!("failed to read config file: {:?}", config_file_path))?;

    // 尝试将用户配置解析到一个临时的 Config 结构体中
    // 注意：这里我们使用 `serde_overwrite` 或手动字段覆盖。
    // 由于 `toml` 和 `serde` 本身不直接提供深度合并功能，
    // 最常见的方式是先加载默认，再加载用户，然后用户的值覆盖默认的值。
    //
    // 如果您想实现更细粒度的合并（例如合并数组或嵌套表），
    // 可能需要更复杂的逻辑或使用专门的库，如 `merge` crate。
    //
    // 对于您当前的结构体，简单的反序列化会将用户提供的字段覆盖默认值。
    // 对于没有提供的字段，它会保留 `config` (默认配置) 中的值。
    // 这正是 `serde(default)` 在字段级别起作用的方式。

    // 为了实现覆盖，您可以先从默认配置开始，然后用用户文件中的数据更新它。
    // `toml::from_str_into` 这样的方法在某些toml库中可能存在，但更直接的方式是：
    // 1. 反序列化用户文件到 Config 结构体。
    // 2. 如果某个字段用户没有提供，那么它会使用该字段的默认值 (如果字段上有 `#[serde(default)]`)。
    // 3. 如果您想保留默认配置的某些子字段，而用户只提供部分，这会变得复杂。
    //
    // 最简单和常用的方法是：
    // let user_config: Config = toml::from_str(&content)
    //     .with_context(|| format!("fail to parse config file '{}'", config_file_path.display()))?;
    // config.app = user_config.app; // 这样会完全替换 App，而不是合并
    // config.i18n = user_config.i18n;
    // config.ui = user_config.ui;

    // 更好的方法是使用 `#[serde(flatten)]` 和多个结构体来处理默认值和用户覆盖，
    // 或者利用 `toml` + `serde` 的 `default` 属性。
    // 您目前的代码，只要 `App`, `I18n`, `Ui` 的字段都有 `#[serde(default)]` 或其子结构体有默认值，
    // 并且这些结构体本身可以从 `Raw` 转换，那么直接解析用户配置就可以了。

    // 重新审视您的结构体和 `#[serde(default)]`：
    // `Config` 的字段 `app`, `i18n`, `ui` 都有 `#[serde(default)]`。
    // 这意味着如果用户配置文件中缺少这些顶层字段，它们会使用 `Config` 的默认实现。
    // `Ui` 的 `filter` 也有 `#[serde(default)]`。
    // `App` 和 `I18n` 自己需要 `Default` 实现。
    // `App` 上的 `#[serde(try_from = "validate::app::Raw")]` 表示它会从 `Raw` 转换。
    //
    // 鉴于此，最直接的解决方案是直接解析用户配置文件，让 `serde(default)` 处理缺失的字段。
    // 这样就避免了 `duplicate key` 的问题。

    let final_config: Config = toml::from_str(&content)
        .with_context(|| {
            format!(
                "fail to parse config file '{}'",
                config_file_path.display()
            )
        })?;

    Ok(final_config)
}
