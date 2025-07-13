use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Widget {
    pub id: String,
    pub class: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Channel {
    pub id: String,
    pub data_type: String,
    pub send: Socket,
    pub recv: Socket,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Socket {
    pub widget_id: String,
    pub socket_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Layout {
    pub entry: String,
    pub areas: Vec<Area>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Area {
    pub id: String,
    pub ratio: u16,
    pub widget_id: Option<String>,
    // vertical or horizontal
    pub direction: Option<String>,
    // id of constraints
    pub constraints: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instance {
    pub id: String,
    pub widgets: Vec<Widget>,
    pub channels: Vec<Channel>,
    pub layout: Layout,
}
