use crate::widget::{ChannelDataType, ControlCode, Filter, Open, Widget, WidgetClass};
use std::str::FromStr;
use std::{collections::HashMap, sync::mpsc};

struct Pipeline {
    widgets: HashMap<String, Box<dyn Widget>>,
    control_code_receivers: HashMap<String, mpsc::Receiver<ControlCode>>,
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
        }
    }
}
