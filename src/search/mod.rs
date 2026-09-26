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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::FilterTranslator;

  fn book(title: &str, authors: &[&str]) -> Book {
    Book {
      title: title.to_string(),
      authors: authors.iter().map(|author| author.to_string()).collect(),
      formats: vec!["EPUB".to_string()],
      ..Book::default()
    }
  }

  fn library() -> Vec<Book> {
    vec![
      book("Harry Potter", &["J. K. Rowling"]),
      book("Преступление и наказание", &["Фёдор Достоевский"]),
      book("きゃりー", &["Kyary"]),
      book("Kyaa Kyoto", &["Someone"]),
      book("中国历史", &["张三", "李四"]),
      book("Die Blechtrommel für Kinder", &["Günter Grass"]),
      book("L'Étranger", &["Albert Camus"]),
      book("Dune", &["Frank Herbert"]),
      book("", &[]),
    ]
  }

  fn search_with(translators: Vec<FilterTranslator>) -> BookSearch {
    let config = FilterConfig {
      translators,
      ..FilterConfig::default()
    };
    BookSearch::new(&library(), &config, BookField::ALL.to_vec())
  }

  fn all_translators() -> Vec<FilterTranslator> {
    vec![
      FilterTranslator::ChinesePinyin,
      FilterTranslator::JapaneseRomaji,
      FilterTranslator::GermanLatin,
      FilterTranslator::FrenchLatin,
      FilterTranslator::RussianLatin,
    ]
  }

  fn ranges(result: &SearchResult, field: BookField) -> Vec<TokenRange> {
    result.highlights.ranges(field).collect()
  }

  #[test]
  fn terms_are_anded_across_fields() {
    let mut search = search_with(vec![]);
    let results = search.search("harry rowling");
    assert_eq!(results.len(), 1);
    assert_eq!(ranges(&results[0], BookField::Title), [(0, 5)]);
    assert_eq!(ranges(&results[0], BookField::Authors), [(4, 11)]);
    assert_eq!(results[0].rank, 0);
    assert!(search.search("harry dune").is_empty());
  }

  #[test]
  fn empty_query_matches_everything_without_highlights() {
    let mut search = search_with(vec![]);
    let results = search.search("   ");
    assert_eq!(results.len(), library().len());
    assert!(
      results
        .iter()
        .all(|result| result.rank == BookField::ALL.len())
    );
    assert!(
      results
        .iter()
        .all(|result| result.highlights == Highlights::default())
    );
  }

  #[test]
  fn transliterated_matches_highlight_source_chars() {
    let mut search = search_with(all_translators());

    let results = search.search("prestup");
    assert_eq!(results.len(), 1);
    assert_eq!(ranges(&results[0], BookField::Title), [(0, 7)]);

    // "kya" covers き and ゃ, two source chars.
    let results = search.search("kyari");
    assert_eq!(results[0].book_index, 2);
    assert_eq!(ranges(&results[0], BookField::Title), [(0, 3)]);

    // Pinyin spans whole characters; authors are joined with " & ".
    let results = search.search("lisi");
    assert_eq!(ranges(&results[0], BookField::Authors), [(3, 5)]);

    let results = search.search("fuer etranger");
    assert!(results.is_empty());
    let results = search.search("fuer");
    assert_eq!(ranges(&results[0], BookField::Title), [(15, 18)]);
    let results = search.search("etrang");
    assert_eq!(ranges(&results[0], BookField::Title), [(2, 8)]);
  }

  #[test]
  fn rank_is_first_matching_field_in_layout_order() {
    let mut search = BookSearch::new(
      &library(),
      &FilterConfig::default(),
      vec![BookField::Authors, BookField::Title],
    );
    let results = search.search("kyary");
    assert_eq!(results[0].rank, 0);
    let results = search.search("dune");
    assert_eq!(results[0].rank, 1);
  }

  #[test]
  fn incremental_search_matches_full_search() {
    let queries = [
      "", "k", "ky", "kya", "kyar", "き", "きゃ", "っ", "っか", "d", "du", "dune", "dune ",
      "dune f", "h", "ha", "an", "ang", "zhong", "zhongg", "f", "fu", "fue", "fuer", "ü", "p",
      "pr", "pre", "pres", "прес", "e", "et", "é", "ét", "l'", "l'é",
    ];
    let mut incremental = search_with(all_translators());
    for query in queries {
      let narrowed = incremental.search(query);
      let full = search_with(all_translators()).search(query);
      assert_eq!(narrowed, full, "query {query:?}");
    }
  }

  /// The examples promised in doc/search.md.
  #[test]
  fn documented_examples_match() {
    let books = [
      "Harry Potter",
      "中国历史",
      "上海",
      "きゃりー",
      "Преступление",
      "für",
      "L'Étranger",
    ]
    .map(|title| book(title, &[]));
    let config = FilterConfig {
      translators: all_translators(),
      ..FilterConfig::default()
    };
    let mut search = BookSearch::new(&books, &config, vec![BookField::Title]);
    for (query, title) in [
      ("harrypot", "Harry Potter"),
      ("zhongguo", "中国历史"),
      ("guoli", "中国历史"),
      ("shanhai", "上海"),
      ("kyari", "きゃりー"),
      ("prestuplenie", "Преступление"),
      ("fuer", "für"),
      ("etranger", "L'Étranger"),
      ("letranger", "L'Étranger"),
    ] {
      let found = search
        .search(query)
        .iter()
        .map(|result| books[result.book_index].title.as_str())
        .collect::<Vec<_>>();
      assert_eq!(found, [title], "{query:?}");
    }
  }

  #[test]
  fn small_kana_after_digraph_is_not_treated_as_refinement() {
    // "き" -> "ki", but "きゃ" -> "kya": "kya" does not contain "ki".
    let mut search = search_with(vec![FilterTranslator::JapaneseRomaji]);
    let books = search
      .search("き")
      .iter()
      .map(|result| result.book_index)
      .collect::<Vec<_>>();
    assert_eq!(books, [2, 5]);
    let books = search
      .search("きゃ")
      .iter()
      .map(|result| result.book_index)
      .collect::<Vec<_>>();
    assert_eq!(books, [2, 3]);
  }
}
