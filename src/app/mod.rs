pub mod filter;
pub mod open;
use anyhow::Result;
use crate::ui::filter::Handler as UiHandler;
use crate::utils::book::Uuids;
use crate::{i18n::filter::Handler as I18nHandler, utils::book::Books};
use ratatui::{Terminal, backend::Backend, widgets::TableState};

pub trait Ui {
    fn tick<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()>;
}
pub struct Filter<'a> {
    books_info: filter::BooksInfo,
    filtered_uuids: Uuids,
    books_highlights: filter::BooksHighlights,
    pub table_state: TableState,
    input: String,
    should_quit: bool,
    exit_on_open: bool,
    i18n_handler: I18nHandler,
    ui_handler: UiHandler,
    database: &'a Books,
}
