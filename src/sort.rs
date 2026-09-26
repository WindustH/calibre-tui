use crate::library::{Book, BookField};
use crate::search::SearchResult;
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

/// A sort spec resolved against the library: every book's position in the
/// fully sorted library, so results sort by integer comparison.
pub struct BookOrder {
  spec: SortSpec,
  positions: Vec<usize>,
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
        .inspect(|_| index += 1)
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
  pub fn parse(input: &str) -> Option<Self> {
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

impl BookOrder {
  /// Sort the whole library by the spec's keys (ASCII case-insensitive),
  /// breaking ties by library order.
  pub fn new(spec: SortSpec, books: &[Book]) -> Self {
    let keys = spec
      .keys
      .iter()
      .map(|key| {
        let values = books
          .iter()
          .map(|book| book.field_text(key.field).to_ascii_lowercase())
          .collect::<Vec<_>>();
        (values, key.direction)
      })
      .collect::<Vec<_>>();

    let mut order = (0..books.len()).collect::<Vec<_>>();
    order.sort_by(|&left, &right| {
      keys
        .iter()
        .map(|(values, direction)| {
          let ordering = values[left].cmp(&values[right]);
          match direction {
            SortDirection::Asc => ordering,
            SortDirection::Desc => ordering.reverse(),
          }
        })
        .find(|ordering| ordering.is_ne())
        .unwrap_or(Ordering::Equal)
    });

    let mut positions = vec![0; books.len()];
    for (position, book_index) in order.into_iter().enumerate() {
      positions[book_index] = position;
    }
    Self { spec, positions }
  }

  pub fn spec(&self) -> &SortSpec {
    &self.spec
  }

  /// Group results by match rank (layout field priority), then order each
  /// group by the sort spec.
  pub fn sort(&self, results: &mut [SearchResult]) {
    results.sort_unstable_by_key(|result| (result.rank, self.positions[result.book_index]));
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::search::Highlights;

  fn book(title: &str, author: &str) -> Book {
    Book {
      title: title.to_string(),
      authors: vec![author.to_string()],
      ..Book::default()
    }
  }

  fn result(book_index: usize, rank: usize) -> SearchResult {
    SearchResult {
      book_index,
      rank,
      highlights: Highlights::default(),
    }
  }

  fn sorted(order: &BookOrder, mut results: Vec<SearchResult>) -> Vec<usize> {
    order.sort(&mut results);
    results.iter().map(|result| result.book_index).collect()
  }

  #[test]
  fn sorts_by_keys_then_library_order() {
    let books = [
      book("b", "Y"),
      book("A", "Z"),
      book("a", "X"),
      book("c", "X"),
    ];
    let all = || (0..books.len()).map(|index| result(index, 0)).collect();

    let order = BookOrder::new(SortSpec::default(), &books);
    assert_eq!(sorted(&order, all()), [1, 2, 0, 3]);

    let spec = SortSpec::parse(&["authors", "title", "desc"]).unwrap();
    assert_eq!(spec.label(), "authors asc, title desc");
    let order = BookOrder::new(spec, &books);
    assert_eq!(sorted(&order, all()), [3, 2, 0, 1]);
  }

  #[test]
  fn match_rank_groups_before_sort_keys() {
    let books = [book("a", ""), book("b", ""), book("c", "")];
    let order = BookOrder::new(SortSpec::default(), &books);
    let results = vec![result(0, 1), result(2, 0), result(1, 0)];
    assert_eq!(sorted(&order, results), [1, 2, 0]);
  }

  #[test]
  fn parse_rejects_unknown_fields_and_empty_args() {
    assert!(SortSpec::parse(&[]).is_err());
    assert!(SortSpec::parse(&["rating"]).is_err());
    assert_eq!(
      SortSpec::parse(&["Title", "DESCENDING"]).unwrap().label(),
      "title desc"
    );
  }
}
