use crate::i18n::filter::TString;
use crate::utils::book::{Books, Uuids};
use anyhow::Result;
use ratatui::widgets::TableState;
use std::collections::HashMap;
mod tick;
mod update_filter_result;

#[derive(Debug, Clone)]
// a translated version of a book's info
pub struct Version {
    pub title: TString,
    pub authors: TString,
    pub series: TString,
    pub tags: TString,
}

// a book's info of all versions
type Info = HashMap<String, Version>;
// all books's info of all versions
pub type BooksInfo = HashMap<String, Info>;

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

impl<'a> super::Filter<'a> {
    // initialize filter command
    pub fn new(
        database: &'a Books,
        i18n_config: &crate::config::i18n::Filter,
        ui_config: &crate::config::ui::Filter,
        exit_on_open: bool,
    ) -> Result<Self> {
        // create i18n handler
        let i18n_handler = crate::i18n::filter::Handler::new(&i18n_config)?;
        let ui_handler = crate::ui::filter::Handler::new(ui_config)?;
        // initialize table state
        let mut table_state = TableState::default();
        if !database.is_empty() {
            table_state.select(Some(0));
        }

        let mut filtered_uuids = Uuids::new();
        let books_highlights = BooksHighlights::new();
        // build filter source
        let mut books_info = BooksInfo::new();

        // iterate through books
        for (uuid, book) in database {
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
            books_highlights,
            books_info,
            filtered_uuids,
            table_state,
            input: String::new(),
            should_quit: false,
            exit_on_open,
            i18n_handler,
            ui_handler,
            database,
        })
    }

    // nagivate up
    fn previous_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if self.filtered_uuids.is_empty() {
                    0
                } else if i == 0 {
                    self.filtered_uuids.len() - 1
                } else {
                    i - 1
                }
            }
            None if !self.filtered_uuids.is_empty() => 0,
            _ => 0,
        };
        self.table_state.select(Some(i));
    }

    // nagivate down
    fn next_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if self.filtered_uuids.is_empty() {
                    0
                } else if i >= self.filtered_uuids.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None if !self.filtered_uuids.is_empty() => 0,
            _ => 0,
        };
        self.table_state.select(Some(i));
    }

    // get the uuids of hovered book
    pub fn get_hovered(&self) -> Option<String> {
        if let Some(hovered_index) = self.table_state.selected() {
            if let Some(uuid) = self.filtered_uuids.get(hovered_index) {
                return Some(uuid.clone());
            }
        };
        None
    }
    // // get user's input in the inputbox
    // pub fn get_input(&self) -> &String {
    //     &self.input
    // }
    // pub fn get_table_state(&self) -> Option<usize> {
    //     self.table_state.selected()
    // }
    // pub fn get_filtered_uuids(&self) -> &Uuids {
    //     &self.filtered_uuids
    // }
    // pub fn get_books_highlights(&self) -> &BooksHighlights {
    //     &self.books_highlights
    // }
}
