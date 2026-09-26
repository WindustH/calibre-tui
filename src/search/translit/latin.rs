use super::{IndexedText, Translator, index_by_char, latin_char};

/// Accent folding used for both `french-latin` and `spanish-latin`.
pub(super) struct LatinTranslator;

impl Translator for LatinTranslator {
  fn index_text(&self, text: &str) -> IndexedText {
    index_by_char(text, latin_char)
  }

  fn normalize_query(&self, query: &str) -> String {
    index_by_char(query, latin_char).into_text()
  }
}
