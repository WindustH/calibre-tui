use super::{IndexedText, Translator, fullwidth_ascii, index_by_char, latin_char};
use anyhow::Result;

pub(super) struct GermanLatinFilter;

impl Translator for GermanLatinFilter {
  fn index_text(&self, text: &str) -> Result<IndexedText> {
    Ok(index_by_char(text, german_char))
  }

  fn normalize_query(&self, query: &str) -> Result<String> {
    Ok(index_by_char(query, german_char).text)
  }
}

fn german_char(ch: char) -> String {
  if let Some(converted) = fullwidth_ascii(ch) {
    return converted.to_string();
  }

  match ch {
    'Ä' | 'ä' => "ae".to_string(),
    'Ö' | 'ö' => "oe".to_string(),
    'Ü' | 'ü' => "ue".to_string(),
    'ẞ' | 'ß' => "ss".to_string(),
    _ => latin_char(ch),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn umlauts_are_searchable_as_ascii() {
    let filter = GermanLatinFilter;
    assert_eq!(
      filter.index_text("Führer Straße").unwrap().text,
      "fuehrerstrasse"
    );
  }

  #[test]
  fn unknown_original_letters_are_retained() {
    let filter = GermanLatinFilter;
    assert_eq!(filter.index_text("猫").unwrap().text, "猫");
  }
}
