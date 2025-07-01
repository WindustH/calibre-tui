use serde::{Deserialize, Serialize};

pub mod inputbox {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Border {
        pub fg: String,
    }
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Title {
        pub fg: String,
    }
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Inputbox {
    pub fg: String,
    pub border: inputbox::Border,
    pub title: inputbox::Title,
}

pub mod table;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Table {
    pub border: table::Border,
    pub title: table::Title,
	pub columns: Vec<table::Column>
}
