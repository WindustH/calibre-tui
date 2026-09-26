use super::{IndexedText, Translator, index_by_char, latin_char};

pub(super) struct GermanLatinTranslator;

impl Translator for GermanLatinTranslator {
  fn index_text(&self, text: &str) -> IndexedText {
    index_by_char(text, german_char)
  }

  fn normalize_query(&self, query: &str) -> String {
    index_by_char(query, german_char).into_text()
  }
}

fn german_char(ch: char, out: &mut String) {
  match ch {
    'Ä' | 'ä' => out.push_str("ae"),
    'Ö' | 'ö' => out.push_str("oe"),
    'Ü' | 'ü' => out.push_str("ue"),
    'ẞ' | 'ß' => out.push_str("ss"),
    _ => latin_char(ch, out),
  }
}
