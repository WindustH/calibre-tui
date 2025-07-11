use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

// uuids of books
pub type Uuids = Vec<String>;
pub type Uuid = String;

// metadata (title, authors, series, tags) for book
// #[derive(Debug, Clone)]
// pub struct Metadata {
//     pub title: String,
//     pub authors: String,
//     pub series: String,
//     pub tags: Vec<String>,
// }

// book struct
#[derive(Debug, Clone)]
pub struct Book {
    pub path: PathBuf,
    pub timestamp: DateTime<Utc>, // time when the book was added to library
    pub pubdate: Option<DateTime<Utc>>, // publish date
    pub last_modified: DateTime<Utc>, // last modified time
    // pub metadata: HashMap<String, Metadata>,
    pub title: String,
    pub authors: Vec<String>,
    pub series: String,
    pub tags: Vec<String>,
}

// books pool using hashmap
pub type Books = HashMap<String, Book>;

// pub fn open_books(database: &Books, uuids: &Uuids) {
//     for uuid in uuids {
//         if let Some(book) = database.get(uuid) {
//             match open::that(PathBuf::from(&book.path)) {
//                 Ok(_) => (),
//                 Err(e) => panic!("encounter error when open books: {:?}", e),
//             }
//         }
//     }
// }
