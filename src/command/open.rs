use crate::utils::book::{Books, Uuids};
use std::path::PathBuf;

pub fn open_books(books: &Books, uuids: &Uuids) {
    for uuid in uuids {
        if let Some(book) = books.get(uuid) {
            open::that(PathBuf::from(book.path.clone()));
        }
    }
}
