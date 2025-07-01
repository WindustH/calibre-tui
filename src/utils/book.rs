use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;


// uuids of books
pub type Uuids=Vec<String>;

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
    pub timestamp: DateTime<Utc>,     // time when the book was added to library
    pub pubdate: Option<DateTime<Utc>>, // publish date
    pub last_modified: DateTime<Utc>, // last modified time
    // pub metadata: HashMap<String, Metadata>,
    pub title: String,
    pub authors: Vec<String>,
    pub series: String,
    pub tags: Vec<String>
}

// books pool using hashmap
#[derive(Debug, Clone)]
pub struct Books(pub HashMap<String, Book>);

// impl Deref to make Books acts like a hashmap
impl Deref for Books {
    type Target = HashMap<String, Book>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Books {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// iterator
impl<'a> IntoIterator for &'a Books {
    type Item = (&'a String, &'a Book);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Book>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// mut iterator
impl<'a> IntoIterator for &'a mut Books {
    type Item = (&'a String, &'a mut Book);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Book>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

/// ownership iterator
impl IntoIterator for Books {
    type Item = (String, Book);
    type IntoIter = std::collections::hash_map::IntoIter<String, Book>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
