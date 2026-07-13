use super::{IndexedText, Translator, index_by_char, latin_char};
use anyhow::Result;

pub(super) struct FrenchLatinTranslator;

impl Translator for FrenchLatinTranslator {
  fn index_text(&self, text: &str) -> Result<IndexedText> {
    Ok(index_by_char(text, french_char))
  }

  fn normalize_query(&self, query: &str) -> Result<String> {
    Ok(index_by_char(query, french_char).text)
  }
}

fn french_char(ch: char) -> String {
  latin_char(ch)
}
