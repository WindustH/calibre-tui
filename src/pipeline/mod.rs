use crate::config::pipeline::Area;
use crate::widget::{ChannelDataType, ControlCode, Filter, Open, Ui, Widget, WidgetClass};
use anyhow::Result;
use ratatui::Terminal;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use std::io::Stdout;
use std::str::FromStr;
use std::{collections::HashMap, sync::mpsc};
struct Pipeline {
    widgets: HashMap<String, Box<dyn Widget>>,
    control_code_receivers: HashMap<String, mpsc::Receiver<ControlCode>>,
    config: crate::config::pipeline::Instance,
}

impl Pipeline {
    pub fn new(config: &crate::config::Config, instance_id: &str) -> Self {
        let mut widgets = HashMap::<String, Box<dyn Widget>>::new();

        // get the instance by id, if not found, panic
        let instance_config = match config
            .pipeline
            .instances
            .iter()
            .find(|i| i.id == instance_id)
        {
            Some(instance) => instance,
            None => {
                panic!("instance with id {} not found", instance_id);
            }
        };

        // initialize widgets
        for widget_config in &instance_config.widgets {
            let widget_class = match WidgetClass::from_str(&widget_config.class) {
                Ok(class) => class,
                Err(_) => {
                    panic!("widget class {} not found", widget_config.class);
                }
            };
            let widget: Box<dyn Widget> = match widget_class {
                // find filter widget
                WidgetClass::Filter => Box::new(match Filter::new(config, false) {
                    Ok(filter) => filter,
                    Err(err) => {
                        panic!("failed to create filter widget: {}", err);
                    }
                }),
                // find open widget
                WidgetClass::Open => Box::new(Open::new(config.app.library_path.clone())),
            };
            widgets.insert(widget_config.id.clone(), widget);
        }

        // initialize channels
        for channel_config in &instance_config.channels {
            let channel_data_type = match ChannelDataType::from_str(&channel_config.data_type) {
                Ok(data_type) => data_type,
                Err(_) => {
                    panic!("channel data type {} not found", channel_config.data_type);
                }
            };
            // try to get send and recv widgets
            let send = match widgets.get(&channel_config.send.widget_id) {
                Some(widget) => widget,
                None => {
                    panic!("widget with id {} not found", channel_config.send.widget_id);
                }
            };
            let recv = match widgets.get(&channel_config.recv.widget_id) {
                Some(widget) => widget,
                None => {
                    panic!("widget with id {} not found", channel_config.recv.widget_id);
                }
            };
            // check if the channel data type is correct
            match send.get_socket_type(&channel_config.send.socket_id) {
                Ok(data_type) => {
                    if data_type != channel_data_type {
                        panic!(
                            "channel {} data type mismatch: expected {:?}, got {:?}",
                            channel_config.id, channel_data_type, data_type
                        );
                    }
                }
                Err(err) => {
                    panic!(
                        "failed to get socket type for widget {}: {}",
                        &channel_config.send.widget_id, err
                    );
                }
            }
            match recv.get_socket_type(&channel_config.recv.socket_id) {
                Ok(data_type) => {
                    if data_type != channel_data_type {
                        panic!(
                            "channel {} data type mismatch: expected {:?}, got {:?}",
                            channel_config.id, channel_data_type, data_type
                        );
                    }
                }
                Err(err) => {
                    panic!(
                        "failed to get socket type for widget {}: {}",
                        &channel_config.recv.widget_id, err
                    );
                }
            }
            match channel_data_type {
                ChannelDataType::Uuid => {
                    // create a channel for uuid
                    let (sender, receiver) = mpsc::channel::<String>();
                    // connect the send widget
                    if let Err(err) = send.connect(
                        &channel_config.id,
                        &channel_config.send.socket_id,
                        Box::new(sender),
                    ) {
                        panic!("failed to connect send widget: {}", err);
                    }
                    // connect the recv widget
                    if let Err(err) = recv.connect(
                        &channel_config.id,
                        &channel_config.recv.socket_id,
                        Box::new(receiver),
                    ) {
                        panic!("failed to connect recv widget: {}", err);
                    }
                }
                _ => {}
            }
        }

        Self {
            widgets,
            control_code_receivers: HashMap::<String, mpsc::Receiver<ControlCode>>::new(),
            config: instance_config.clone(),
        }
    }
    fn tick(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        for widget in self.widgets.values() {
            // do tick for widget
            widget.tick()?;
        }

        let entry_area = match self
            .config
            .layout
            .areas
            .iter()
            .find(|a| a.id == self.config.layout.entry)
        {
            Some(area) => area,
            None => {
                panic!("entry area with id {} not found", self.config.layout.entry);
            }
        };
        // recursively iterate through areas in the layout
        let mut area_stack: Vec<Area> = vec![entry_area.clone()];
        let mut rect_stack: Vec<Rect> = vec![terminal.size()?];

        while !area_stack.is_empty() {
            // handle the area on top of the stack
            let area = match area_stack.pop() {
                Some(area) => area,
                None => continue,
            };
            // get sub areas of this area
            if let Some(constraints) = area.constraints {
                // if constraints is not empty, push sub areas to the stack

                // get the respective rect from the stack
                let rect = match rect_stack.pop() {
                    Some(rect) => rect,
                    None => continue,
                };
                let mut sub_areas: Vec<Area> = vec![];

                // iterate through sub areas id
                for sub_area_id in &constraints {
                    if let Some(sub_area) = self
                        .config
                        .layout
                        .areas
                        .iter()
                        .find(|a| a.id == *sub_area_id)
                    {
                        // push the sub area to the stack
                        area_stack.push(sub_area.clone());
                        // save the sub areas for chunk-splitting later
                        sub_areas.push(sub_area.clone());
                    } else {
                        panic!("sub area with id {} not found", sub_area_id);
                    }
                }
                // get the direction from config
                let direction = match area.direction {
                    Some(dir) => match dir.as_str() {
                        "horizontal" => ratatui::layout::Direction::Horizontal,
                        "vertical" => ratatui::layout::Direction::Vertical,
                        _ => panic!("invalid direction: {}", dir),
                    },
                    None => ratatui::layout::Direction::Vertical,
                };
                // get chunks
                let chunks = Layout::default()
                    .direction(direction)
                    .constraints(
                        sub_areas
                            .iter()
                            .map(|a| ratatui::layout::Constraint::Percentage(a.ratio))
                            .collect::<Vec<_>>(),
                    )
                    .split(rect);

                // push the chunks to the stack
                for chunk in chunks.iter().rev() {
                    rect_stack.push(*chunk);
                }
            } else {
                // this is a leaf area, draw the widget in this area
                let widget_id = match area.widget_id {
                    Some(id) => id,
                    None => {
                        panic!(
                            "area with id {} has no widget_id and isn't a parent area",
                            area.id
                        );
                    }
                };
                if let Some(widget) = self.widgets.get(&widget_id) {
                    // draw the widget in the rect

                    // try to downcast the widget to Ui trait
                    if let Some(ui_widget) = widget.as_any().downcast_ref::<dyn Ui>() {
                        ui_widget.draw_tick(terminal, rect_stack.pop().unwrap())?;
                    } else {
                        panic!("widget with id {} is not a Ui widget", widget_id);
                    }
                } else {
                    panic!("widget with id {} not found", widget_id);
                }
            }
        }

        Ok(())
    }
}
