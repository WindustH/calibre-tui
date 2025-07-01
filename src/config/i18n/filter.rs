use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Pinyin {
    pub enabled: bool,
    pub fuzzy_enabled: bool,
    pub fuzzy_groups: Vec<Vec<String>>,
}
