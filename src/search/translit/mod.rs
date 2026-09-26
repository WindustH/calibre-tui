//! Search text normalization: the plain lowercase index plus optional
//! transliterations that let Latin-script queries match other scripts.

use crate::config::{FilterConfig, FilterTranslator};

mod german;
mod japanese;
mod latin;
mod pinyin;
mod russian;

/// Normalized search text whose tokens map back to the source text: token
/// `i` is produced by the `i`-th non-whitespace char of the source and may
/// be empty (for example dropped punctuation).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexedText {
  text: String,
  /// Byte offset in `text` where each token ends.
  token_ends: Vec<usize>,
}

impl IndexedText {
  #[cfg(test)]
  pub fn text(&self) -> &str {
    &self.text
  }

  fn into_text(self) -> String {
    self.text
  }

  fn push_token(&mut self, token: &str) {
    self.text.push_str(token);
    self.token_ends.push(self.text.len());
  }

  /// Find the first occurrence of `query` and return the source token range
  /// `(start, end)` it covers, end-exclusive.
  pub fn find(&self, query: &str) -> Option<(usize, usize)> {
    if query.is_empty() {
      return None;
    }
    let start = self.text.find(query)?;
    let end = start + query.len();
    let first = self
      .token_ends
      .partition_point(|&token_end| token_end <= start);
    let last = self
      .token_ends
      .partition_point(|&token_end| token_end < end);
    Some((first, last + 1))
  }
}

pub trait Translator {
  fn index_text(&self, text: &str) -> IndexedText;
  fn normalize_query(&self, query: &str) -> String;
}

pub struct Translators {
  translators: Vec<Box<dyn Translator>>,
}

impl Translators {
  pub fn from_config(config: &FilterConfig) -> Self {
    let mut enabled = Vec::new();
    let mut translators: Vec<Box<dyn Translator>> = Vec::new();

    for &translator in &config.translators {
      // French and Spanish share the same accent folding.
      let translator = match translator {
        FilterTranslator::SpanishLatin => FilterTranslator::FrenchLatin,
        other => other,
      };
      if enabled.contains(&translator) {
        continue;
      }
      enabled.push(translator);
      translators.push(match translator {
        FilterTranslator::ChinesePinyin => Box::new(pinyin::ChinesePinyinTranslator::new(config)),
        FilterTranslator::JapaneseRomaji => Box::new(japanese::JapaneseRomajiTranslator),
        FilterTranslator::GermanLatin => Box::new(german::GermanLatinTranslator),
        FilterTranslator::FrenchLatin | FilterTranslator::SpanishLatin => {
          Box::new(latin::LatinTranslator)
        }
        FilterTranslator::RussianLatin => Box::new(russian::RussianLatinTranslator),
      });
    }

    Self { translators }
  }

  pub fn index_texts<'a>(&'a self, text: &'a str) -> impl Iterator<Item = IndexedText> + 'a {
    self
      .translators
      .iter()
      .map(move |translator| translator.index_text(text))
  }

  pub fn normalize_queries(&self, query: &str) -> Vec<String> {
    self
      .translators
      .iter()
      .map(|translator| translator.normalize_query(query))
      .collect()
  }
}

/// Lowercase, whitespace-free index of the original text.
pub fn index_plain_text(text: &str) -> IndexedText {
  let mut indexed = IndexedText::default();
  for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
    indexed.text.extend(ch.to_lowercase());
    indexed.token_ends.push(indexed.text.len());
  }
  indexed
}

pub fn normalize_plain_query(query: &str) -> String {
  query
    .chars()
    .filter(|ch| !ch.is_whitespace())
    .flat_map(char::to_lowercase)
    .collect()
}

fn index_by_char(text: &str, transliterate: fn(char, &mut String)) -> IndexedText {
  let mut indexed = IndexedText::default();
  let mut translated = String::new();
  for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
    translated.clear();
    transliterate(ch, &mut translated);
    push_search_text(&mut indexed.text, ch, &translated);
    indexed.token_ends.push(indexed.text.len());
  }
  indexed
}

/// Append the searchable form of `translated` (the transliteration of
/// `source`): its ASCII letters and digits, lowercased; if it has none,
/// `source` itself lowercased, unless `source` is punctuation or was
/// transliterated to nothing.
fn push_search_text(out: &mut String, source: char, translated: &str) {
  let start = out.len();
  out.extend(
    translated
      .chars()
      .filter(char::is_ascii_alphanumeric)
      .map(|ch| ch.to_ascii_lowercase()),
  );
  if out.len() == start && !translated.is_empty() && source.is_alphanumeric() {
    out.extend(source.to_lowercase());
  }
}

/// Fold accented Latin letters and full-width ASCII to plain ASCII.
fn latin_char(ch: char, out: &mut String) {
  if let Some(converted) = fullwidth_ascii(ch) {
    out.push(converted);
    return;
  }

  let folded = match ch {
    'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å'
    | 'ā' | 'ă' | 'ą' => "a",
    'Æ' | 'æ' => "ae",
    'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' | 'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c",
    'Ð' | 'Ď' | 'Đ' | 'ð' | 'ď' | 'đ' => "d",
    'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' | 'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ'
    | 'ė' | 'ę' | 'ě' => "e",
    'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' | 'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g",
    'Ĥ' | 'Ħ' | 'ĥ' | 'ħ' => "h",
    'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' | 'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī'
    | 'ĭ' | 'į' | 'ı' => "i",
    'Ĵ' | 'ĵ' => "j",
    'Ķ' | 'ķ' => "k",
    'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' | 'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l",
    'Ñ' | 'Ń' | 'Ņ' | 'Ň' | 'ñ' | 'ń' | 'ņ' | 'ň' => "n",
    'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø'
    | 'ō' | 'ŏ' | 'ő' => "o",
    'Œ' | 'œ' => "oe",
    'Ŕ' | 'Ŗ' | 'Ř' | 'ŕ' | 'ŗ' | 'ř' => "r",
    'Ś' | 'Ŝ' | 'Ş' | 'Š' | 'ś' | 'ŝ' | 'ş' | 'š' => "s",
    'Ţ' | 'Ť' | 'Ŧ' | 'ţ' | 'ť' | 'ŧ' => "t",
    'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' | 'ù' | 'ú' | 'û' | 'ü' | 'ũ'
    | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u",
    'Ŵ' | 'ŵ' => "w",
    'Ý' | 'Ŷ' | 'Ÿ' | 'ý' | 'ÿ' | 'ŷ' => "y",
    'Ź' | 'Ż' | 'Ž' | 'ź' | 'ż' | 'ž' => "z",
    'Þ' | 'þ' => "th",
    _ => {
      out.extend(ch.to_lowercase());
      return;
    }
  };
  out.push_str(folded);
}

fn fullwidth_ascii(ch: char) -> Option<char> {
  match ch {
    '０'..='９' => char::from_u32((ch as u32 - '０' as u32) + '0' as u32),
    'Ａ'..='Ｚ' => char::from_u32((ch as u32 - 'Ａ' as u32) + 'A' as u32),
    'ａ'..='ｚ' => char::from_u32((ch as u32 - 'ａ' as u32) + 'a' as u32),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn find_maps_byte_matches_back_to_source_tokens() {
    let mut text = IndexedText::default();
    for token in ["zhong", "guo", "", "li", "shi"] {
      text.push_token(token);
    }
    // "ngg" spans the first two syllables.
    assert_eq!(text.find("ngg"), Some((0, 2)));
    // An empty token (dropped punctuation) is never the start of a match.
    assert_eq!(text.find("li"), Some((3, 4)));
    assert_eq!(text.find("guoli"), Some((1, 4)));
    assert_eq!(text.find(""), None);
    assert_eq!(text.find("x"), None);
  }

  #[test]
  fn plain_index_tokens_are_non_whitespace_source_chars() {
    let indexed = index_plain_text(" Ab İ c ");
    assert_eq!(indexed.text(), "abi\u{307}c");
    // `İ` lowercases to two chars but stays a single token.
    assert_eq!(indexed.find("i\u{307}c"), Some((2, 4)));
    assert_eq!(normalize_plain_query("A b"), "ab");
  }

  #[test]
  fn french_and_spanish_share_one_translator() {
    let config = FilterConfig {
      translators: vec![
        FilterTranslator::FrenchLatin,
        FilterTranslator::SpanishLatin,
        FilterTranslator::GermanLatin,
      ],
      ..FilterConfig::default()
    };
    let translators = Translators::from_config(&config);
    assert_eq!(translators.normalize_queries("Niñez"), ["ninez", "ninez"]);
  }
}
