use ratatui::layout::Rect;

use crate::utils::db::load_books_from_db;
use crate::widget::{ChannelDataType, Filter, Open, Widget, WidgetClass};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, sync::mpsc};

pub mod event_tick;
pub mod update_ui_rects;

pub struct Pipeline {
    pub widgets: Arc<HashMap<String, Arc<dyn Widget>>>,
    config: crate::config::pipeline::Instance,
    pub ui_rects: Arc<Mutex<HashMap<String, Rect>>>,
    pub should_exit: Arc<Mutex<bool>>,
}

impl Pipeline {
    pub fn new(config: &crate::config::Config, instance_id: &str) -> Self {
        let mut widgets = HashMap::<String, Arc<dyn Widget>>::new();
        let books = Arc::new(match load_books_from_db(&config.app.library_path) {
            Ok(books) => books,
            Err(err) => {
                panic!("failed to load books from db: {}", err);
            }
        });

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
            let widget: Arc<dyn Widget> = match widget_class {
                // find filter widget
                WidgetClass::Filter => Arc::new(match Filter::new(config, Arc::clone(&books)) {
                    Ok(filter) => filter,
                    Err(err) => {
                        panic!("failed to create filter widget: {}", err);
                    }
                }),
                // find open widget
                WidgetClass::Open => Arc::new(Open::new(Arc::clone(&books))),
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
                    panic!(
                        "sender widget with id {} not found",
                        channel_config.send.widget_id
                    );
                }
            };
            let recv = match widgets.get(&channel_config.recv.widget_id) {
                Some(widget) => widget,
                None => {
                    panic!(
                        "receiver widget with id {} not found",
                        channel_config.recv.widget_id
                    );
                }
            };
            // check if the channel data type is correct
            match send.get_socket_type(&channel_config.send.socket_id) {
                Ok(data_type) => {
                    if data_type != channel_data_type {
                        panic!(
                            "channel {} data type mismatch: expected {:?}, got {:?} from sender widget {}",
                            channel_config.id,
                            channel_data_type,
                            data_type,
                            &channel_config.send.widget_id
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
                            "channel {} data type mismatch: expected {:?}, got {:?} from receiver widget {}",
                            channel_config.id,
                            channel_data_type,
                            data_type,
                            &channel_config.recv.widget_id
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
            widgets: Arc::new(widgets),
            config: instance_config.clone(),
            should_exit: Arc::new(Mutex::new(false)),
            ui_rects: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
