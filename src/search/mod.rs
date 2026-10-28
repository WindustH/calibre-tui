//! In-memory book search: normalized per-field indexes, AND matching of
//! whitespace-separated terms, and highlight ranges for display.

mod translit;

use crate::config::FilterConfig;
use crate::library::{Book, BookField};
use std::iter;
use translit::{IndexedText, Translators, index_plain_text, normalize_plain_query};

/// Range over the non-whitespace chars of a field's display text
/// ([`Book::field_text`]), end-exclusive.
pub type TokenRange = (usize, usize);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Highlights {
  /// Sorted by field, then start; ranges of one field never overlap or touch.
  ranges: Vec<(BookField, TokenRange)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
  pub book_index: usize,
  /// Layout position of the first searchable field that matched; the number
  /// of searchable fields when nothing was matched (empty query).
  pub rank: usize,
  pub highlights: Highlights,
}

pub struct BookSearch {
  /// Searchable fields in layout (priority) order.
  fields: Vec<BookField>,
  /// `books[book_index][slot]` indexes `fields[slot]`.
  books: Vec<Vec<IndexedField>>,
  translators: Translators,
  previous: Option<PreviousSearch>,
}

struct IndexedField {
  plain: IndexedText,
  /// One entry per translator; `None` when identical to `plain`.
  translated: Vec<Option<IndexedText>>,
}

/// A query term normalized like the index: `plain`, then one form per translator.
struct QueryTerm {
  plain: String,
  translated: Vec<String>,
}

struct PreviousSearch {
  terms: Vec<QueryTerm>,
  matches: Vec<usize>,
}

impl BookSearch {
  pub fn new(books: &[Book], config: &FilterConfig, fields: Vec<BookField>) -> Self {
    let translators = Translators::from_config(config);
    let books = books
      .iter()
      .map(|book| {
        fields
          .iter()
          .map(|&field| IndexedField::new(&book.field_text(field), &translators))
          .collect()
      })
      .collect();
    Self {
      fields,
      books,
      translators,
      previous: None,
    }
  }

  /// Books matching every whitespace-separated term of `query`, in library
  /// order. An empty query matches every book.
  pub fn search(&mut self, query: &str) -> Vec<SearchResult> {
    let terms = self.query_terms(query);
    // A refinement of the previous query (typically one more typed char) can
    // only drop books, so only the previous matches need rescanning.
    let candidates = match self.previous.take() {
      Some(previous) if refines(&terms, &previous.terms) => previous.matches,
      _ => (0..self.books.len()).collect(),
    };
    let results = candidates
      .into_iter()
      .filter_map(|book_index| self.match_book(book_index, &terms))
      .collect::<Vec<_>>();
    self.previous = Some(PreviousSearch {
      terms,
      matches: results.iter().map(|result| result.book_index).collect(),
    });
    results
  }

  fn query_terms(&self, query: &str) -> Vec<QueryTerm> {
    query
      .split_whitespace()
      .map(|term| QueryTerm {
        plain: normalize_plain_query(term),
        translated: self.translators.normalize_queries(term),
      })
      .collect()
  }

  fn match_book(&self, book_index: usize, terms: &[QueryTerm]) -> Option<SearchResult> {
    let fields = &self.books[book_index];
    let mut highlights = Highlights::default();
    let mut rank = self.fields.len();
    for term in terms {
      let mut matched = false;
      for (slot, (&field, indexed)) in self.fields.iter().zip(fields).enumerate() {
        if let Some(range) = indexed.find(term) {
          highlights.ranges.push((field, range));
          rank = rank.min(slot);
          matched = true;
        }
      }
      if !matched {
        return None;
      }
    }
    highlights.normalize();
    Some(SearchResult {
      book_index,
      rank,
      highlights,
    })
  }
}

impl Highlights {
  pub fn ranges(&self, field: BookField) -> impl Iterator<Item = TokenRange> + '_ {
    self
      .ranges
      .iter()
      .filter(move |(range_field, _)| *range_field == field)
      .map(|&(_, range)| range)
  }

  fn normalize(&mut self) {
    self.ranges.sort_unstable();
    let mut merged: Vec<(BookField, TokenRange)> = Vec::with_capacity(self.ranges.len());
    for (field, (start, end)) in self.ranges.drain(..) {
      match merged.last_mut() {
        Some((last_field, last)) if *last_field == field && start <= last.1 => {
          last.1 = last.1.max(end);
        }
        _ => merged.push((field, (start, end))),
      }
    }
    self.ranges = merged;
  }
}

impl IndexedField {
  fn new(text: &str, translators: &Translators) -> Self {
    let plain = index_plain_text(text);
    let translated = translators
      .index_texts(text)
      .map(|indexed| (indexed != plain).then_some(indexed))
      .collect();
    Self { plain, translated }
  }

  /// First match of the term: plain text first, then each translator.
  fn find(&self, term: &QueryTerm) -> Option<TokenRange> {
    self.plain.find(&term.plain).or_else(|| {
      self
        .translated
        .iter()
        .zip(&term.translated)
        .find_map(|(indexed, query)| match indexed {
          Some(indexed) => indexed.find(query),
          None if *query != term.plain => self.plain.find(query),
          None => None,
        })
    })
  }
}

impl QueryTerm {
  fn forms(&self) -> impl Iterator<Item = &str> {
    iter::once(self.plain.as_str()).chain(self.translated.iter().map(String::as_str))
  }

  /// Whether any text containing a form of `self` also contains the
  /// corresponding form of `other`. Empty forms never match anything.
  fn implies(&self, other: &Self) -> bool {
    self
      .forms()
      .zip(other.forms())
      .all(|(new, old)| new.is_empty() || (!old.is_empty() && new.contains(old)))
  }
}

/// Whether every book matching `new` also matches `old`.
fn refines(new: &[QueryTerm], old: &[QueryTerm]) -> bool {
  old
    .iter()
    .all(|old_term| new.iter().any(|new_term| new_term.implies(old_term)))
}
