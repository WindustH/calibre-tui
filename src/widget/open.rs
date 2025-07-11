use crate::utils::book::Uuid;
use crate::utils::db::get_book_by_uuid_from_db;
use crate::widget::{ChannelDataType, Open, Widget};
use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc;
use strum_macros::{Display, EnumString};

impl Open {
    fn new(library_path: PathBuf) -> Result<Self> {
        Ok(Self {
            library_path,
            receivers: HashMap::new(),
            // status_code_senders: HashMap::new(),
        })
    }
}

#[derive(Debug, EnumString, Display, PartialEq)]
enum Socket {
    #[strum(serialize = "recv-uuid-to-open")]
    RecvUuidToOpen,
}

impl Widget for Open {
    fn tick(&mut self) -> Result<()> {
        // iterate through all receivers
        for (_, receiver) in &self.receivers {
            // iterate through all messages in the receiver
            for msg in receiver {
                if let Some(book) = get_book_by_uuid_from_db(&self.library_path, &msg)? {
                    match open::that(PathBuf::from(&book.path)) {
                        Ok(_) => (),
                        Err(e) => panic!("encounter error when open books: {:?}", e),
                    }
                } else {
                    Err(anyhow::anyhow!("failed to get book by uuid: {}", msg))?;
                }
            }
        }
        Ok(())
    }

    fn connect(&mut self, channel_id: &str, socket_id: &str, plug: Box<dyn Any>) -> Result<()> {
        match Socket::from_str(socket_id)? {
            Socket::RecvUuidToOpen => {
                if let Ok(receiver) = plug.downcast::<mpsc::Receiver<Uuid>>() {
                    // check if the channel_name already exists
                    if self.receivers.contains_key(channel_id) {
                        Err(anyhow::anyhow!("channel {} already exists", channel_id))?;
                    } else {
                        // insert the receiver into the receivers map
                        self.receivers.insert(channel_id.to_string(), *receiver);
                    }
                } else {
                    Err(anyhow::anyhow!("plug is not a mpsc::Receiver<String>"))?;
                }
                Ok(())
            }
        }
    }
    fn get_socket_type(&self, socket_id: &str) -> Result<ChannelDataType> {
        match Socket::from_str(socket_id)? {
            Socket::RecvUuidToOpen => Ok(ChannelDataType::Uuid),
        }
    }
}
