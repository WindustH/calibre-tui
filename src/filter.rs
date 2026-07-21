use crate::config::FilterConfig;
use crate::i18n::filter::{IndexedText, Translators, index_plain_text, normalize_plain_query};
use crate::layout::{BookField, Layout};
use crate::utils::book::Book;
use anyhow::{Result, anyhow};

pub type HighlightRanges = Vec<(usize, usize)>;

#[derive(Debug, Clone, Default)]
pub struct BookHighlights {
  pub title: HighlightRanges,
  pub authors: HighlightRanges,
  pub series: HighlightRanges,
  pub formats: HighlightRanges,
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
  search_fields: Vec<BookField>,
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
  formats: IndexedField,
  tags: IndexedField,
}

#[derive(Debug, Clone)]
struct IndexedField {
  versions: Vec<IndexedText>,
}

impl BookSearch {
  pub fn new(books: &[Book], config: &FilterConfig, layout: &Layout) -> Result<Self> {
    let translators = Translators::from_config(config)?;
    let search_fields = layout.search_fields().collect::<Vec<_>>();
    let books = books
      .iter()
      .enumerate()
      .map(|(book_index, book)| {
        Ok(IndexedBook {
          book_index,
          title: index_field(&book.title, &translators)?,
          authors: index_field(&book.authors.join(" & "), &translators)?,
          series: index_field(&book.series, &translators)?,
          formats: index_field(&book.formats.join(", "), &translators)?,
          tags: index_field(&book.tags.join(", "), &translators)?,
        })
      })
      .collect::<Result<Vec<_>>>()?;

    Ok(Self {
      books,
      translators,
      search_fields,
    })
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
        match match_book_term(book, &self.search_fields, term)? {
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
  pub fn ranges(&self, field: BookField) -> &HighlightRanges {
    match field {
      BookField::Title => &self.title,
      BookField::Authors => &self.authors,
      BookField::Series => &self.series,
      BookField::Formats => &self.formats,
      BookField::Tags => &self.tags,
    }
  }

  fn is_empty(&self) -> bool {
    self.title.is_empty()
      && self.authors.is_empty()
      && self.series.is_empty()
      && self.formats.is_empty()
      && self.tags.is_empty()
  }

  fn extend(&mut self, other: Self) {
    self.title.extend(other.title);
    self.authors.extend(other.authors);
    self.series.extend(other.series);
    self.formats.extend(other.formats);
    self.tags.extend(other.tags);
  }

  fn normalize(&mut self) {
    normalize_ranges(&mut self.title);
    normalize_ranges(&mut self.authors);
    normalize_ranges(&mut self.series);
    normalize_ranges(&mut self.formats);
    normalize_ranges(&mut self.tags);
  }

  fn extend_field(&mut self, field: BookField, ranges: HighlightRanges) {
    match field {
      BookField::Title => self.title.extend(ranges),
      BookField::Authors => self.authors.extend(ranges),
      BookField::Series => self.series.extend(ranges),
      BookField::Formats => self.formats.extend(ranges),
      BookField::Tags => self.tags.extend(ranges),
    }
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

fn match_book_term(
  book: &IndexedBook,
  search_fields: &[BookField],
  term: &QueryTerm,
) -> Result<Option<BookHighlights>> {
  let mut highlights = BookHighlights::default();
  for field in search_fields {
    highlights.extend_field(
      *field,
      match_field(book.field(*field), &term.versions)?.unwrap_or_default(),
    );
  }

  if highlights.is_empty() {
    Ok(None)
  } else {
    highlights.normalize();
    Ok(Some(highlights))
  }
}

impl IndexedBook {
  fn field(&self, field: BookField) -> &IndexedField {
    match field {
      BookField::Title => &self.title,
      BookField::Authors => &self.authors,
      BookField::Series => &self.series,
      BookField::Formats => &self.formats,
      BookField::Tags => &self.tags,
    }
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
