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
	pub ratio: Option<u16>,
    pub widget_id: Option<String>,
    pub left_right: Option<(Box<Layout>, Box<Layout>)>,
    pub up_down: Option<(Box<Layout>, Box<Layout>)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instance {
    pub widgets: Vec<Widget>,
    pub channels: Vec<Channel>,
    pub layout: Layout,
}
