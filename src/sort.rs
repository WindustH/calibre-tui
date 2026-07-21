use crate::filter::SearchResult;
use crate::layout::{BookField, Layout};
use crate::utils::book::Book;
use anyhow::{Result, bail};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
  Asc,
  Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortKey {
  pub field: BookField,
  pub direction: SortDirection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortSpec {
  keys: Vec<SortKey>,
}

impl Default for SortSpec {
  fn default() -> Self {
    Self {
      keys: vec![SortKey {
        field: BookField::Title,
        direction: SortDirection::Asc,
      }],
    }
  }
}

impl SortSpec {
  pub fn parse(args: &[&str]) -> Result<Self> {
    if args.is_empty() {
      bail!("usage: sort <field> [asc|desc] [field] [asc|desc] ...");
    }

    let mut keys = Vec::new();
    let mut index = 0;
    while index < args.len() {
      let Some(field) = BookField::parse(args[index]) else {
        bail!("unknown sort field: {}", args[index]);
      };
      index += 1;

      let direction = args
        .get(index)
        .and_then(|value| SortDirection::parse(value))
        .map(|direction| {
          index += 1;
          direction
        })
        .unwrap_or(SortDirection::Asc);

      keys.push(SortKey { field, direction });
    }

    Ok(Self { keys })
  }

  pub fn label(&self) -> String {
    self
      .keys
      .iter()
      .map(|key| format!("{} {}", key.field.name(), key.direction.name()))
      .collect::<Vec<_>>()
      .join(", ")
  }
}

impl SortDirection {
  fn parse(input: &str) -> Option<Self> {
    match input.to_ascii_lowercase().as_str() {
      "asc" | "ascending" => Some(Self::Asc),
      "desc" | "descending" => Some(Self::Desc),
      _ => None,
    }
  }

  fn name(self) -> &'static str {
    match self {
      Self::Asc => "asc",
      Self::Desc => "desc",
    }
  }
}

pub fn sort_results(
  results: &mut [SearchResult],
  books: &[Book],
  spec: &SortSpec,
  layout: &Layout,
) {
  let match_fields = layout.search_fields().collect::<Vec<_>>();
  results.sort_by(|left, right| {
    compare_match_priority(left, right, &match_fields)
      .then_with(|| compare_results(left, right, books, spec))
  });
}

fn compare_match_priority(
  left: &SearchResult,
  right: &SearchResult,
  match_fields: &[BookField],
) -> Ordering {
  match_priority(left, match_fields).cmp(&match_priority(right, match_fields))
}

fn match_priority(result: &SearchResult, match_fields: &[BookField]) -> usize {
  match_fields
    .iter()
    .position(|field| !result.highlights.ranges(*field).is_empty())
    .unwrap_or(match_fields.len())
}

fn compare_results(
  left: &SearchResult,
  right: &SearchResult,
  books: &[Book],
  spec: &SortSpec,
) -> Ordering {
  let Some(left_book) = books.get(left.book_index) else {
    return left.book_index.cmp(&right.book_index);
  };
  let Some(right_book) = books.get(right.book_index) else {
    return left.book_index.cmp(&right.book_index);
  };

  for key in &spec.keys {
    let ordering = field_value(left_book, key.field).cmp(&field_value(right_book, key.field));
    let ordering = match key.direction {
      SortDirection::Asc => ordering,
      SortDirection::Desc => ordering.reverse(),
    };
    if !ordering.is_eq() {
      return ordering;
    }
  }

  left.book_index.cmp(&right.book_index)
}

fn field_value(book: &Book, field: BookField) -> String {
  match field {
    BookField::Title => book.title.to_ascii_lowercase(),
    BookField::Authors => book.authors.join(" & ").to_ascii_lowercase(),
    BookField::Series => book.series.to_ascii_lowercase(),
    BookField::Formats => book.formats.join(", ").to_ascii_lowercase(),
    BookField::Tags => book.tags.join(", ").to_ascii_lowercase(),
  }
}
