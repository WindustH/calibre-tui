use crate::utils::book::{Books, Uuids};
use std::path::PathBuf;

pub fn open_books(database: &Books, uuids: &Uuids) {
    for uuid in uuids {
        if let Some(book) = database.get(uuid) {
            match open::that(PathBuf::from(&book.path)) {
                Ok(_) => (),
                Err(e) => panic!("encounter error when open books: {:?}", e),
            }
        }
    }
}
