use crate::config::FilterConfig;
use crate::i18n::filter::{IndexedText, Translators, index_plain_text, normalize_plain_query};
use crate::utils::book::Book;
use anyhow::{Result, anyhow};

pub type HighlightRanges = Vec<(usize, usize)>;

#[derive(Debug, Clone, Default)]
pub struct BookHighlights {
  pub title: HighlightRanges,
  pub authors: HighlightRanges,
  pub series: HighlightRanges,
  pub tags: HighlightRanges,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
  pub book_index: usize,
  pub highlights: BookHighlights,
}

pub struct BookSearch {
  books: Vec<IndexedBook>,
  translators: Translators,
}

struct QueryTerm {
  versions: Vec<String>,
}

#[derive(Debug, Clone)]
struct IndexedBook {
  book_index: usize,
  title: IndexedField,
  authors: IndexedField,
  series: IndexedField,
  tags: IndexedField,
}

#[derive(Debug, Clone)]
struct IndexedField {
  versions: Vec<IndexedText>,
}

impl BookSearch {
  pub fn new(books: &[Book], config: &FilterConfig) -> Result<Self> {
    let translators = Translators::from_config(config)?;
    let books = books
      .iter()
      .enumerate()
      .map(|(book_index, book)| {
        Ok(IndexedBook {
          book_index,
          title: index_field(&book.title, &translators)?,
          authors: index_field(&book.authors.join(" & "), &translators)?,
          series: index_field(&book.series, &translators)?,
          tags: index_field(&book.tags.join(", "), &translators)?,
        })
      })
      .collect::<Result<Vec<_>>>()?;

    Ok(Self { books, translators })
  }

  pub fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
    if query.trim().is_empty() {
      return Ok(
        self
          .books
          .iter()
          .map(|book| SearchResult {
            book_index: book.book_index,
            highlights: BookHighlights::default(),
          })
          .collect(),
      );
    }

    let terms = self.query_terms(query)?;

    let mut results = Vec::new();
    for book in &self.books {
      let mut highlights = BookHighlights::default();
      let mut all_terms_matched = true;

      for term in &terms {
        match match_book_term(book, term)? {
          Some(term_highlights) => highlights.extend(term_highlights),
          None => {
            all_terms_matched = false;
            break;
          }
        }
      }

      if all_terms_matched {
        highlights.normalize();
        if !highlights.is_empty() {
          results.push(SearchResult {
            book_index: book.book_index,
            highlights,
          });
        }
      }
    }

    Ok(results)
  }

  fn query_terms(&self, query: &str) -> Result<Vec<QueryTerm>> {
    query
      .split_whitespace()
      .map(|term| {
        let mut versions = vec![normalize_plain_query(term)];
        versions.extend(self.translators.normalize_queries(term)?);
        Ok(QueryTerm { versions })
      })
      .collect()
  }
}

impl BookHighlights {
  fn is_empty(&self) -> bool {
    self.title.is_empty()
      && self.authors.is_empty()
      && self.series.is_empty()
      && self.tags.is_empty()
  }

  fn extend(&mut self, other: Self) {
    self.title.extend(other.title);
    self.authors.extend(other.authors);
    self.series.extend(other.series);
    self.tags.extend(other.tags);
  }

  fn normalize(&mut self) {
    normalize_ranges(&mut self.title);
    normalize_ranges(&mut self.authors);
    normalize_ranges(&mut self.series);
    normalize_ranges(&mut self.tags);
  }
}

fn index_field(text: &str, translators: &Translators) -> Result<IndexedField> {
  let mut versions = vec![index_plain_text(text)];
  versions.extend(translators.index_texts(text)?);
  Ok(IndexedField { versions })
}

fn match_field(field: &IndexedField, queries: &[String]) -> Result<Option<HighlightRanges>> {
  for (version, query) in field.versions.iter().zip(queries) {
    if let Some(range) = match_text(version, query)? {
      return Ok(Some(vec![range]));
    }
  }

  Ok(None)
}

fn match_book_term(book: &IndexedBook, term: &QueryTerm) -> Result<Option<BookHighlights>> {
  let mut highlights = BookHighlights {
    title: match_field(&book.title, &term.versions)?.unwrap_or_default(),
    authors: match_field(&book.authors, &term.versions)?.unwrap_or_default(),
    series: match_field(&book.series, &term.versions)?.unwrap_or_default(),
    tags: match_field(&book.tags, &term.versions)?.unwrap_or_default(),
  };

  if highlights.is_empty() {
    Ok(None)
  } else {
    highlights.normalize();
    Ok(Some(highlights))
  }
}

fn normalize_ranges(ranges: &mut HighlightRanges) {
  ranges.sort_unstable_by_key(|range| range.0);

  let mut merged: HighlightRanges = Vec::new();
  for (start, end) in ranges.drain(..) {
    let Some(last) = merged.last_mut() else {
      merged.push((start, end));
      continue;
    };

    if start <= last.1 {
      last.1 = last.1.max(end);
    } else {
      merged.push((start, end));
    }
  }

  *ranges = merged;
}

fn match_text(text: &IndexedText, query: &str) -> Result<Option<(usize, usize)>> {
  if query.is_empty() {
    return Ok(None);
  }

  let Some((match_start_byte, matched)) = text.text.match_indices(query).next() else {
    return Ok(None);
  };

  let start_char = text.text[..match_start_byte].chars().count();
  let end_char = start_char + matched.chars().count();
  if end_char == 0 {
    return Ok(None);
  }

  let char_to_token = text
    .token_bounds
    .windows(2)
    .enumerate()
    .flat_map(|(token_index, bounds)| vec![token_index; bounds[1] - bounds[0]])
    .collect::<Vec<_>>();

  if text.text.chars().count() != char_to_token.len() {
    return Err(anyhow!(
      "search index length mismatch: text has {} chars but token map has {} chars",
      text.text.chars().count(),
      char_to_token.len()
    ));
  }

  if end_char > char_to_token.len() {
    return Ok(None);
  }

  let start_token = char_to_token[start_char];
  let end_token = char_to_token[end_char - 1] + 1;

  Ok(Some((start_token, end_token)))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::{FilterConfig, FilterTranslator};
  use std::path::PathBuf;

  #[test]
  fn space_separated_terms_are_matched_with_and_across_fields() {
    let books = vec![Book {
      path: PathBuf::from("/tmp/book.epub"),
      title: "dianzi shebei".to_string(),
      authors: vec!["someone".to_string()],
      series: String::new(),
      tags: vec!["jishu".to_string()],
    }];
    let search = BookSearch::new(&books, &plain_config()).unwrap();

    assert_eq!(search.search("dianzi jishu").unwrap().len(), 1);
    assert_eq!(search.search("jishu dianzi").unwrap().len(), 1);
    assert!(search.search("dianzi missing").unwrap().is_empty());
  }

  fn plain_config() -> FilterConfig {
    FilterConfig {
      translators: Vec::new(),
      pinyin_fuzzy: false,
      pinyin_fuzzy_groups: Vec::new(),
    }
  }

  #[test]
  fn multiple_translators_can_match_translated_and_original_text() {
    let books = vec![
      test_book("Führer Straße"),
      test_book("Преступление"),
      test_book("ガッコウ"),
    ];
    let search = BookSearch::new(
      &books,
      &translator_config(vec![
        FilterTranslator::GermanLatin,
        FilterTranslator::RussianLatin,
        FilterTranslator::Romaji,
      ]),
    )
    .unwrap();

    assert_eq!(search.search("fuehrer").unwrap().len(), 1);
    assert_eq!(search.search("prestuplenie").unwrap().len(), 1);
    assert_eq!(search.search("gakkou").unwrap().len(), 1);
    assert_eq!(search.search("Преступление").unwrap().len(), 1);
  }

  fn translator_config(translators: Vec<FilterTranslator>) -> FilterConfig {
    FilterConfig {
      translators,
      pinyin_fuzzy: false,
      pinyin_fuzzy_groups: Vec::new(),
    }
  }

  fn test_book(title: &str) -> Book {
    Book {
      path: PathBuf::from("/tmp/book.epub"),
      title: title.to_string(),
      authors: Vec::new(),
      series: String::new(),
      tags: Vec::new(),
    }
  }
}
