pub mod filter;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filter{
    pub inputbox: filter::Inputbox,
    pub table: filter::Table
}