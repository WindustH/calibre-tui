use super::{IndexedText, Translator, index_by_char, latin_char};
use anyhow::Result;

pub(super) struct SpanishLatinTranslator;

impl Translator for SpanishLatinTranslator {
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
