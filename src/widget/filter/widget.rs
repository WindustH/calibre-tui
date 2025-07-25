use crate::widget::{ChannelDataType, Filter, Ui, Widget};
use anyhow::Result;
use std::str::FromStr;
use strum_macros::{Display, EnumString};

#[derive(Debug, EnumString, Display, PartialEq)]
enum Socket {
    #[strum(serialize = "send-selected-uuid")]
    SendSelectedUuid,
    #[strum(serialize = "send-hovered-uuid")]
    SendHoverdUuid,
}

impl Widget for Filter {
    fn tick(&self) -> Result<()> {
        Ok(())
    }
    fn connect(
        &self,
        channel_id: &str,
        socket_id: &str,
        plug: Box<dyn std::any::Any>,
    ) -> Result<()> {
        match Socket::from_str(socket_id)? {
            Socket::SendSelectedUuid => {
                if let Ok(sender) = plug.downcast::<std::sync::mpsc::Sender<String>>() {
                    // check if the channel_name already exists
                    if self
                        .selected_uuid_senders
                        .lock()
                        .unwrap()
                        .contains_key(channel_id)
                    {
                        return Err(anyhow::anyhow!("channel {} already exists", channel_id));
                    } else {
                        // insert the sender into the selected_senders map
                        self.selected_uuid_senders
                            .lock()
                            .unwrap()
                            .insert(channel_id.to_string(), *sender);
                    }
                } else {
                    return Err(anyhow::anyhow!("plug is not a mpsc::Sender<String>"));
                }
            }
            Socket::SendHoverdUuid => {
                if let Ok(sender) = plug.downcast::<std::sync::mpsc::Sender<String>>() {
                    // check if the channel_name already exists
                    if self
                        .hovered_uuid_senders
                        .lock()
                        .unwrap()
                        .contains_key(channel_id)
                    {
                        return Err(anyhow::anyhow!("channel {} already exists", channel_id));
                    } else {
                        // insert the sender into the hovered_senders map
                        self.hovered_uuid_senders
                            .lock()
                            .unwrap()
                            .insert(channel_id.to_string(), *sender);
                    }
                } else {
                    return Err(anyhow::anyhow!("plug is not a mpsc::Sender<String>"));
                }
            }
        }
        Ok(())
    }
    fn get_socket_type(&self, socket_id: &str) -> Result<ChannelDataType> {
        match Socket::from_str(socket_id)? {
            Socket::SendHoverdUuid => Ok(ChannelDataType::Uuid),
            Socket::SendSelectedUuid => Ok(ChannelDataType::Uuid),
        }
    }
    fn as_ui(&self) -> Option<&dyn Ui> {
        Some(self)
    }
}
