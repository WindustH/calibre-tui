use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]

pub struct Column {
    pub label: String,

    pub fg: String,

    pub hovered_fg: String,
    pub hovered_bg: String,

    pub highlighted_fg: String,
    pub hovered_highlighted_fg: String,

    pub label_fg: String,

    pub ratio: u16,
    pub position: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(try_from = "crate::config::validate::ui::filter::table::RawColumns")]
pub struct Columns(pub(crate) Vec<Column>);

impl<'a> IntoIterator for &'a Columns {
    type Item = &'a Column;
    type IntoIter = std::slice::Iter<'a, Column>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Border {
    pub fg: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Title {
    pub fg: String,
}
