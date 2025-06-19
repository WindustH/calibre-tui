use crate::config::Config;
use crate::db::load_books_from_db;
use crate::pinyin::{build_canonical_map, to_canonical_pinyin};
use anyhow::Result;
use ratatui::widgets::TableState;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Book {
    pub title: String,
    pub author: String,
    pub series: String,
    pub tags: String,
    pub path: String,
    pub title_pinyin: String,
    pub author_pinyin: String,
    pub series_pinyin: String,
    pub tags_pinyin: String,
}

pub struct App {
    pub all_books: Vec<Book>,
    pub filtered_books: Vec<Book>,
    pub table_state: TableState,
    pub input: String,
    pub library_path: PathBuf,
    pub should_quit: bool,
    pub canonical_map: HashMap<String, String>,
    pub config: Config,
    pub exit_on_open: bool,
}

impl App {
    pub fn new(library_path: PathBuf, config: Config, exit_on_open: bool) -> Result<Self> {
        let all_books = load_books_from_db(&library_path, config.pinyin_search_enabled)?;
        let filtered_books = all_books.clone();
        let mut table_state = TableState::default();
        if !filtered_books.is_empty() {
            table_state.select(Some(0));
        }

        let canonical_map = build_canonical_map(&config.fuzzy_pinyin);

        Ok(Self {
            all_books,
            filtered_books,
            table_state,
            input: String::new(),
            library_path,
            should_quit: false,
            canonical_map,
            config,
            exit_on_open,
        })
    }

    pub fn filter_books(&mut self) {
        if self.input.is_empty() {
            self.filtered_books = self.all_books.clone();
            if !self.filtered_books.is_empty() {
                self.table_state.select(Some(0));
            } else {
                self.table_state.select(None);
            }
            return;
        }

        let input_lower_with_spaces = self.input.to_lowercase();

        self.filtered_books = self
            .all_books
            .iter()
            .filter(|book| {
                if book.title.to_lowercase().contains(&input_lower_with_spaces)
                    || book.author.to_lowercase().contains(&input_lower_with_spaces)
                    || book.series.to_lowercase().contains(&input_lower_with_spaces)
                    || book.tags.to_lowercase().contains(&input_lower_with_spaces)
                {
                    return true;
                }

                // Only perform pinyin search if it's enabled in the config.
                if self.config.pinyin_search_enabled {
                    let input_lower_no_spaces = input_lower_with_spaces.replace(' ', "");
                    if !input_lower_no_spaces.is_empty() {
                        let canonical_query = to_canonical_pinyin(&input_lower_no_spaces, &self.canonical_map);

                        let canonical_title = to_canonical_pinyin(&book.title_pinyin, &self.canonical_map);
                        let canonical_author =
                            to_canonical_pinyin(&book.author_pinyin, &self.canonical_map);
                        let canonical_series =
                            to_canonical_pinyin(&book.series_pinyin, &self.canonical_map);
                        let canonical_tags = to_canonical_pinyin(&book.tags_pinyin, &self.canonical_map);

                        return canonical_title.contains(&canonical_query)
                            || canonical_author.contains(&canonical_query)
                            || canonical_series.contains(&canonical_query)
                            || canonical_tags.contains(&canonical_query);
                    }
                }

                false
            })
            .cloned()
            .collect();

        if !self.filtered_books.is_empty() {
            self.table_state.select(Some(0));
        } else {
            self.table_state.select(None);
        }
    }

    pub fn previous_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if self.filtered_books.is_empty() {
                    0
                } else if i == 0 {
                    self.filtered_books.len() - 1
                } else {
                    i - 1
                }
            }
            None if !self.filtered_books.is_empty() => 0,
            _ => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn next_item(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if self.filtered_books.is_empty() {
                    0
                } else if i >= self.filtered_books.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None if !self.filtered_books.is_empty() => 0,
            _ => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn open_selected_book(&mut self) {
        if let Some(selected_index) = self.table_state.selected() {
            if let Some(book) = self.filtered_books.get(selected_index) {
                let full_path = self.library_path.join(&book.path);
                if open::that(&full_path).is_ok() {
                    if self.exit_on_open {
                        self.should_quit = true;
                    }
                }
            }
        }
    }
}
