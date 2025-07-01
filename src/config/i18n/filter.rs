use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pinyin {
    pub enabled: bool,
    pub fuzzy_enabled: bool,
    pub fuzzy_groups: Vec<Vec<String>>,
}
