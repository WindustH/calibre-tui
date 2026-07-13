use super::{IndexedText, Translator, fullwidth_ascii, index_by_char, latin_char};
use anyhow::Result;

pub(super) struct GermanLatinTranslator;

impl Translator for GermanLatinTranslator {
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
