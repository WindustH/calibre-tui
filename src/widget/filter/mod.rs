use crate::i18n::filter::TString;
use crate::utils::book::{Books, Uuids};
use crate::widget::Filter;
use anyhow::{Context, Result};
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
mod ui;
mod update;
mod widget;

#[derive(Debug, Clone)]
// a translated version of a book's info
// to pass through filter
pub(super) struct Version {
    pub title: TString,
    pub authors: TString,
    pub series: TString,
    pub tags: TString,
}

// a book's info of all versions
type Info = HashMap<String, Version>;
// all books's info of all versions
pub(super) type BooksInfo = HashMap<String, Info>;

/// highlights of a string
/// Vec<(usize, usize)> is the array of start and end index
type Highlights = Vec<(bool, usize, usize)>;
// highlights of a book's info
pub struct BookHighlights {
    pub title: Highlights,
    pub authors: Highlights,
    pub series: Highlights,
    pub tags: Highlights,
}
// highlights of all books' info
// the string is uuid
pub type BooksHighlights = HashMap<String, BookHighlights>;

impl Filter {
    // initialize filter command
    pub fn new(config: &crate::config::Config, books: Arc<Books>) -> Result<Self> {
        // create i18n handler
        let i18n_handler = crate::i18n::filter::Handler::new(&config.i18n.filter)
            .context("failed to create i18n handler")?;
        let ui_handler = crate::ui::filter::Handler::new(&config.ui.filter)
            .context("failed to create ui handler")?;
        // initialize table state
        let mut table_state = TableState::default();

        if !books.is_empty() {
            table_state.select(Some(0));
        }

        let mut filtered_uuids = Uuids::new();
        let books_highlights = BooksHighlights::new();
        // build filter source
        let mut books_info = BooksInfo::new();

        // iterate through books
        for (uuid, book) in books.iter() {
            let mut versions = Info::new();
            // prebuild tags and authors into string
            let authors_str = book.authors.join(" & ");
            let tags_str = book.tags.join(", ");

            // generate default version
            let process_str = |input: &str| {
                let processed = input.to_lowercase().replace(" ", "");
                let ranges: Vec<usize> = (0..=processed.chars().count()).collect();
                (processed, ranges)
            };

            let default_variant = Version {
                title: process_str(&book.title),
                series: process_str(&book.series),
                tags: process_str(&tags_str),
                authors: process_str(&authors_str),
            };
            versions.insert("default".to_string(), default_variant);

            // generate versions
            for (name, translator) in &i18n_handler.translators {
                if translator.is_enabled() {
                    let translation_results = (
                        translator.trans_book_info(&book.title.replace(" ", "").to_lowercase()),
                        translator.trans_book_info(&book.series.replace(" ", "").to_lowercase()),
                        translator.trans_book_info(&tags_str.replace(" ", "").to_lowercase()),
                        translator.trans_book_info(&authors_str.replace(" ", "").to_lowercase()),
                    );

                    // deal with all cases
                    match translation_results {
                        (Ok(title), Ok(series), Ok(tags), Ok(authors)) => {
                            versions.insert(
                                name.to_string(),
                                Version {
                                    title,
                                    authors,
                                    series,
                                    tags,
                                },
                            );
                        }
                        (Err(e), _, _, _) => println!("Error translating title: {:?}", e),
                        (_, Err(e), _, _) => println!("Error translating series: {:?}", e),
                        (_, _, Err(e), _) => println!("Error translating tags: {:?}", e),
                        (_, _, _, Err(e)) => println!("Error translating authors: {:?}", e),
                    };
                }
            }

            books_info.insert(uuid.to_string(), versions);
            filtered_uuids.push(uuid.to_string());
        }
        Ok(Self {
            books,
            books_highlights: Arc::new(Mutex::new(books_highlights)),
            books_info,
            filtered_uuids: Arc::new(Mutex::new(filtered_uuids)),
            table_state: Arc::new(Mutex::new(table_state)),
            input: Arc::new(Mutex::new(String::new())),
            i18n_handler,
            ui_handler,
            selected_uuid_senders: Arc::new(Mutex::new(HashMap::new())),
            hovered_uuid_senders: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    // nagivate up
    fn previous_item(&self) {
        let i = match self.table_state.lock().unwrap().selected() {
            Some(i) => {
                if self.filtered_uuids.lock().unwrap().is_empty() {
                    0
                } else if i == 0 {
                    self.filtered_uuids.lock().unwrap().len() - 1
                } else {
                    i - 1
                }
            }
            None if !self.filtered_uuids.lock().unwrap().is_empty() => 0,
            _ => 0,
        };
        self.table_state.lock().unwrap().select(Some(i));
    }

    // nagivate down
    fn next_item(&self) {
        let i = match self.table_state.lock().unwrap().selected() {
            Some(i) => {
                if self.filtered_uuids.lock().unwrap().is_empty() {
                    0
                } else if i >= self.filtered_uuids.lock().unwrap().len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None if !self.filtered_uuids.lock().unwrap().is_empty() => 0,
            _ => 0,
        };
        self.table_state.lock().unwrap().select(Some(i));
    }

    // get the uuids of hovered book
    // pub fn get_hovered(&self) -> Option<String> {
    //     if let Some(hovered_index) = self.table_state.selected() {
    //         if let Some(uuid) = self.filtered_uuids.get(hovered_index) {
    //             return Some(uuid.clone());
    //         }
    //     };
    //     None
    // }

    pub fn send_selected_uuid(&self, uuid: String) -> Result<()> {
        // send selected uuid to all senders
        for sender in self.selected_uuid_senders.lock().unwrap().values() {
            sender.send(uuid.clone())?;
        }
        Ok(())
    }
    pub fn send_hovered_uuid(&self, uuid: String) -> Result<()> {
        // send hovered uuid to all senders
        for sender in self.hovered_uuid_senders.lock().unwrap().values() {
            sender.send(uuid.clone())?;
        }
        Ok(())
    }
}
