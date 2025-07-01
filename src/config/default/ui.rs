use crate::config::ui;

impl Default for ui::Filter {
    fn default() -> Self {
        Self {
            inputbox: ui::filter::Inputbox {
                border: ui::filter::inputbox::Border {
                    fg: "Blue".to_string(),
                },
                title: ui::filter::inputbox::Title {
                    fg: "Blue".to_string(),
                },
                fg: "White".to_string(),
            },
            table: ui::filter::Table {
                columns: vec![
                    ui::filter::table::Column {
                        label: "title".to_string(),
                        position: 0,
                        ratio: 40,
                        fg: "White".to_string(),
                        hovered_fg: "White".to_string(),
                        hovered_bg: "Blue".to_string(),
                        label_fg: "Blue".to_string(),
                        highlighted_fg: "Red".to_string(),
                        hovered_highlighted_fg: "Yellow".to_string(),
                    },
                    ui::filter::table::Column {
                        label: "title".to_string(),
                        position: 0,
                        ratio: 40,
                        fg: "Cyan".to_string(),
                        hovered_fg: "White".to_string(),
                        hovered_bg: "Blue".to_string(),
                        label_fg: "Blue".to_string(),
                        highlighted_fg: "Red".to_string(),
                        hovered_highlighted_fg: "Yellow".to_string(),
                    },
                    ui::filter::table::Column {
                        label: "title".to_string(),
                        position: 0,
                        ratio: 40,
                        fg: "White".to_string(),
                        hovered_fg: "White".to_string(),
                        hovered_bg: "Blue".to_string(),
                        label_fg: "Blue".to_string(),
                        highlighted_fg: "Red".to_string(),
                        hovered_highlighted_fg: "Yellow".to_string(),
                    },
                    ui::filter::table::Column {
                        label: "title".to_string(),
                        position: 0,
                        ratio: 40,
                        fg: "Cyan".to_string(),
                        hovered_fg: "White".to_string(),
                        hovered_bg: "Blue".to_string(),
                        label_fg: "Blue".to_string(),
                        highlighted_fg: "Red".to_string(),
                        hovered_highlighted_fg: "Yellow".to_string(),
                    },
                ],
                border: ui::filter::table::Border {
                    fg: "Blue".to_string(),
                },
                title: ui::filter::table::Title {
                    fg: "Blue".to_string(),
                },
            },
        }
    }
}
