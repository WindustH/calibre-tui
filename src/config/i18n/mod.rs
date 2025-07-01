use serde::{Deserialize, Serialize};

pub mod filter;

// intermediate structs for nesting
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filter {
    pub pinyin: filter::Pinyin,
}
