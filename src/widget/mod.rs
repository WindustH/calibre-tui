use crate::ui::filter::Handler as UiHandler;
use crate::utils::book::{Uuid, Uuids};
use crate::{i18n::filter::Handler as I18nHandler, utils::book::Books};
use anyhow::Result;
use ratatui::layout::Rect;
use ratatui::prelude::CrosstermBackend;
use ratatui::{Terminal, backend::Backend, widgets::TableState};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::mpsc;
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

#[derive(Debug, Clone)]
pub enum ControlCode {
    Quit,
    Defocus,
}

pub trait Ui {
    fn draw_tick(&self, terminal: &mut Terminal<CrosstermBackend<Stdout>>,rect: Rect) -> Result<()>;
    fn event_tick(&self) -> Result<()>;
}

/// widget should be able to handle input from
/// other widgets and send output to other widgets
/// in a tick loop
pub trait Widget {
    fn tick(&self) -> Result<()>;
    fn connect(&self, channel_name: &str, socket_name: &str, plug: Box<dyn Any>) -> Result<()>;
    fn get_socket_type(&self, socket_name: &str) -> Result<ChannelDataType>;
    fn as_ui(&self) -> Option<&dyn Ui>;
}

pub struct Filter {
    books_info: filter::BooksInfo,
    // use refcell to support interior mutability
    // change when doing update
    filtered_uuids: RefCell<Uuids>,
    books_highlights: RefCell<filter::BooksHighlights>,

    // change when nagivating
    table_state: RefCell<TableState>,
    // change when got input
    input: RefCell<String>,
    exit_on_open: bool,
    i18n_handler: I18nHandler,
    ui_handler: UiHandler,
    books: Books,
    // mark_signal_receivers: HashMap<String, mpsc::Receiver<String>>,

    // send selected book's uuid to other widgets
    selected_uuid_senders: RefCell<HashMap<String, mpsc::Sender<Uuid>>>,
    // send hovered book's uuid to other widgets
    // when hovered book is changed
    hovered_uuid_senders: RefCell<HashMap<String, mpsc::Sender<Uuid>>>,
    // send control signal to pipeline manager
    control_signal_sender: RefCell<HashMap<String, mpsc::Sender<ControlCode>>>,
    // send status code to other widgets
    // status_code_senders: HashMap<String, mpsc::Sender<StatusCode>>,
}

pub struct Open {
    library_path: PathBuf,
    receivers: RefCell<HashMap<String, mpsc::Receiver<Uuid>>>,
    // send status code to other widgets
    // status_code_senders: HashMap<String, mpsc::Sender<StatusCode>>,
}
