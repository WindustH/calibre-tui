use crate::utils::book::{Books, Uuid};
use crate::widget::{ChannelDataType, Open, Ui, Widget};
use anyhow::Result;
use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex, mpsc};
use strum_macros::{Display, EnumString};

impl Open {
    pub fn new(books: Arc<Books>) -> Self {
        Self {
            books,
            receivers: Arc::new(Mutex::new(HashMap::new())),
            // status_code_senders: HashMap::new(),
        }
    }
}

#[derive(Debug, EnumString, Display, PartialEq)]
enum Socket {
    #[strum(serialize = "recv-uuid-to-open")]
    RecvUuidToOpen,
}

impl Widget for Open {
    fn tick(&self) -> Result<()> {
        // iterate through all receivers
        for (_, receiver) in self.receivers.lock().unwrap().iter_mut() {
            // iterate through all messages in the receiver
            for msg in receiver.try_iter() {
                if let Some(book) = self.books.get(&msg) {
                    // find the path and try to open
                    match open::that(PathBuf::from(&book.path)) {
                        Ok(_) => (),
                        Err(e) => {
                            return Err(anyhow::anyhow!(
                                "encounter error when open books: {:?}",
                                e
                            ));
                        }
                    }
                } else {
                    // can't find the book path
                    return Err(anyhow::anyhow!("failed to get book by uuid: {}", msg));
                }
            }
        }
        Ok(())
    }

    fn connect(&self, channel_id: &str, socket_id: &str, plug: Box<dyn Any>) -> Result<()> {
        match Socket::from_str(socket_id)? {
            Socket::RecvUuidToOpen => {
                if let Ok(receiver) = plug.downcast::<mpsc::Receiver<Uuid>>() {
                    // check if the channel_name already exists
                    if self.receivers.lock().unwrap().contains_key(channel_id) {
                        return Err(anyhow::anyhow!("channel {} already exists", channel_id));
                    } else {
                        // insert the receiver into the receivers map
                        self.receivers
                            .lock()
                            .unwrap()
                            .insert(channel_id.to_string(), *receiver);
                    }
                } else {
                    return Err(anyhow::anyhow!("plug is not a mpsc::Receiver<String>"));
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
    fn as_ui(&self) -> Option<&dyn Ui> {
        None
    }
}
