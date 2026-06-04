use super::{IndexedText, Translator, index_by_char, latin_char};
use anyhow::Result;

pub(super) struct SpanishLatinFilter;

impl Translator for SpanishLatinFilter {
  fn index_text(&self, text: &str) -> Result<IndexedText> {
    Ok(index_by_char(text, spanish_char))
  }

  fn normalize_query(&self, query: &str) -> Result<String> {
    Ok(index_by_char(query, spanish_char).text)
  }
}

fn spanish_char(ch: char) -> String {
  latin_char(ch)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn accents_are_searchable_as_ascii() {
    let filter = SpanishLatinFilter;
    assert_eq!(filter.index_text("Niñez").unwrap().text, "ninez");
  }

  #[test]
  fn unknown_original_letters_are_retained() {
    let filter = SpanishLatinFilter;
    assert_eq!(filter.index_text("猫").unwrap().text, "猫");
  }
}
