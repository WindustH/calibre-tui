use crate::config::Pipeline;
use crate::config::pipeline::{Channel, Instance, Layout, Socket, Widget};
impl Default for Pipeline {
    fn default() -> Self {
        Self {
            instances: vec![Instance {
                layout: Layout {
                    widget_id: Some("filter-0".to_string()),
                    up_down: None,
                    left_right: None,
                    ratio: None,
                },
                widgets: vec![
                    Widget {
                        id: "filter-0".to_string(),
                        class: "filter".to_string(),
                    },
                    Widget {
                        id: "open-0".to_string(),
                        class: "open".to_string(),
                    },
                ],
                channels: vec![Channel {
                    id: "open-filtered-book-channel".to_string(),
                    data_type: "uuid".to_string(),
                    send: Socket {
                        widget_id: "filter-0".to_string(),
                        socket_id: "send-selected-uuid".to_string(),
                    },
                    recv: Socket {
                        widget_id: "open-0".to_string(),
                        socket_id: "recv-uuid-to-open".to_string(),
                    },
                }],
            }],
        }
    }
}
