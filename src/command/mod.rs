pub mod filter;
pub mod open;
use crate::i18n::filter::Handler as I18nHandler;
use crate::ui::filter::Handler as UiHandler;
use crate::utils::book::{Books,Uuids};
use ratatui::{Terminal, widgets::TableState};


pub struct Filter {
    books_info: filter::BooksInfo,
    filtered_uuids: Uuids,
    books_highlights: filter::BooksHighlights,
    table_state: TableState,
    input: String,
    should_quit: bool,
    exit_on_open: bool,
    i18n_handler: I18nHandler,
    ui_handler: UiHandler,
}
