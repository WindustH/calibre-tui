use crate::i18n::filter::Handler as I18nHandler;
use crate::ui::filter::Handler as UiHandler;
use crate::utils::book::{Books, Uuid, Uuids};
use anyhow::Result;
use crossterm::event::Event;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::{Terminal, widgets::TableState};
use std::any::Any;
use std::collections::HashMap;
use std::io::Stdout;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use strum_macros::{Display, EnumString};

pub mod filter;
pub mod open;

// type of data passed through channel
#[derive(Debug, EnumString, Display, PartialEq)]
pub enum ChannelDataType {
    #[strum(serialize = "uuid")]
    Uuid,
    #[strum(serialize = "control-code")]
    ControlCode,
}

#[derive(Debug, EnumString, Display, PartialEq)]
pub enum WidgetClass {
    #[strum(serialize = "filter")]
    Filter,
    #[strum(serialize = "open")]
    Open,
}

pub trait Ui {
    fn draw_tick(
        &self,
        terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
        rect: Rect,
    ) -> Result<()>;
    fn event_tick(&self, event: &Event) -> Result<()>;
}

/// widget should be able to handle input from
/// other widgets and send output to other widgets
/// in a tick loop
pub trait Widget:Send + Sync {
    fn tick(&self) -> Result<()>;
    fn connect(&self, channel_name: &str, socket_name: &str, plug: Box<dyn Any>) -> Result<()>;
    fn get_socket_type(&self, socket_name: &str) -> Result<ChannelDataType>;
    fn as_ui(&self) -> Option<&dyn Ui>;
}

pub struct Filter {
    // be read in mutiple threads
    books: Arc<Books>,
    // only be read when doing widget tick
    books_info: filter::BooksInfo,
    i18n_handler: I18nHandler,
    // only be read when drawing ui
    ui_handler: UiHandler,

    // change when doing widget tick
    filtered_uuids: Arc<Mutex<Uuids>>,
    books_highlights: Arc<Mutex<filter::BooksHighlights>>,
    table_state: Arc<Mutex<TableState>>,
    input: Arc<Mutex<String>>,

    // mark_signal_receivers: HashMap<String, mpsc::Receiver<String>>,

    // send selected book's uuid to other widgets
    selected_uuid_senders: Arc<Mutex<HashMap<String, mpsc::Sender<Uuid>>>>,
    // send hovered book's uuid to other widgets
    // when hovered book is changed
    hovered_uuid_senders: Arc<Mutex<HashMap<String, mpsc::Sender<Uuid>>>>,
    // send control signal to pipeline manager
}

pub struct Open {
    books: Arc<Books>,
    receivers: Arc<Mutex<HashMap<String, mpsc::Receiver<Uuid>>>>,
    // send status code to other widgets
    // status_code_senders: HashMap<String, mpsc::Sender<StatusCode>>,
}
