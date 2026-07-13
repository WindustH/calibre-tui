use crate::config::{FilterConfig, FilterTranslator};
use anyhow::Result;

mod french;
mod german;
mod japanese;
mod pinyin;
mod russian;
mod spanish;

#[derive(Debug, Clone)]
pub struct IndexedText {
  pub text: String,
  pub token_bounds: Vec<usize>,
}

pub trait Translator {
  fn index_text(&self, text: &str) -> Result<IndexedText>;
  fn normalize_query(&self, query: &str) -> Result<String>;
}

pub struct Translators {
  translators: Vec<Box<dyn Translator>>,
}

impl Translators {
  pub fn from_config(config: &FilterConfig) -> Result<Self> {
    let mut translators: Vec<Box<dyn Translator>> = Vec::new();

    for translator in &config.translators {
      match translator {
        FilterTranslator::ChinesePinyin => {
          translators.push(Box::new(pinyin::ChinesePinyinTranslator::new(config)));
        }
        FilterTranslator::JapaneseRomaji => {
          translators.push(Box::new(japanese::JapaneseRomajiTranslator));
        }
        FilterTranslator::GermanLatin => {
          translators.push(Box::new(german::GermanLatinTranslator));
        }
        FilterTranslator::FrenchLatin => {
          translators.push(Box::new(french::FrenchLatinTranslator));
        }
        FilterTranslator::SpanishLatin => {
          translators.push(Box::new(spanish::SpanishLatinTranslator));
        }
        FilterTranslator::RussianLatin => {
          translators.push(Box::new(russian::RussianLatinTranslator));
        }
      }
    }

    Ok(Self { translators })
  }

  pub fn index_texts(&self, text: &str) -> Result<Vec<IndexedText>> {
    self
      .translators
      .iter()
      .map(|translator| translator.index_text(text))
      .collect()
  }

  pub fn normalize_queries(&self, query: &str) -> Result<Vec<String>> {
    self
      .translators
      .iter()
      .map(|translator| translator.normalize_query(query))
      .collect()
  }
}

pub fn index_plain_text(text: &str) -> IndexedText {
  let mut indexed = String::new();
  let mut token_bounds = vec![0];

  for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
    for lower in ch.to_lowercase() {
      indexed.push(lower);
    }
    token_bounds.push(indexed.chars().count());
  }

  IndexedText {
    text: indexed,
    token_bounds,
  }
}

pub fn normalize_plain_query(query: &str) -> String {
  index_plain_text(query).text
}

fn index_by_char(text: &str, transliterate: fn(char) -> String) -> IndexedText {
  let mut indexed = String::new();
  let mut token_bounds = vec![0];

  for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
    indexed.push_str(&translated_search_text(ch, &transliterate(ch)));
    token_bounds.push(indexed.chars().count());
  }

  IndexedText {
    text: indexed,
    token_bounds,
  }
}

fn translated_search_text(source: char, translated: &str) -> String {
  let ascii = ascii_search_text(translated);
  if !ascii.is_empty() {
    return ascii;
  }

  if translated.is_empty() || !source.is_alphanumeric() {
    return String::new();
  }

  source.to_lowercase().collect()
}

fn ascii_search_text(text: &str) -> String {
  text
    .chars()
    .filter_map(|ch| {
      if ch.is_ascii_alphanumeric() {
        Some(ch.to_ascii_lowercase())
      } else {
        None
      }
    })
    .collect()
}

fn latin_char(ch: char) -> String {
  if let Some(converted) = fullwidth_ascii(ch) {
    return converted.to_string();
  }

  match ch {
    'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Ā' | 'Ă' | 'Ą' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å'
    | 'ā' | 'ă' | 'ą' => "a".to_string(),
    'Æ' | 'æ' => "ae".to_string(),
    'Ç' | 'Ć' | 'Ĉ' | 'Ċ' | 'Č' | 'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => "c".to_string(),
    'Ð' | 'Ď' | 'Đ' | 'ð' | 'ď' | 'đ' => "d".to_string(),
    'È' | 'É' | 'Ê' | 'Ë' | 'Ē' | 'Ĕ' | 'Ė' | 'Ę' | 'Ě' | 'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ'
    | 'ė' | 'ę' | 'ě' => "e".to_string(),
    'Ĝ' | 'Ğ' | 'Ġ' | 'Ģ' | 'ĝ' | 'ğ' | 'ġ' | 'ģ' => "g".to_string(),
    'Ĥ' | 'Ħ' | 'ĥ' | 'ħ' => "h".to_string(),
    'Ì' | 'Í' | 'Î' | 'Ï' | 'Ĩ' | 'Ī' | 'Ĭ' | 'Į' | 'İ' | 'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī'
    | 'ĭ' | 'į' | 'ı' => "i".to_string(),
    'Ĵ' | 'ĵ' => "j".to_string(),
    'Ķ' | 'ķ' => "k".to_string(),
    'Ĺ' | 'Ļ' | 'Ľ' | 'Ŀ' | 'Ł' | 'ĺ' | 'ļ' | 'ľ' | 'ŀ' | 'ł' => "l".to_string(),
    'Ñ' | 'Ń' | 'Ņ' | 'Ň' | 'ñ' | 'ń' | 'ņ' | 'ň' => "n".to_string(),
    'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'Ō' | 'Ŏ' | 'Ő' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø'
    | 'ō' | 'ŏ' | 'ő' => "o".to_string(),
    'Œ' | 'œ' => "oe".to_string(),
    'Ŕ' | 'Ŗ' | 'Ř' | 'ŕ' | 'ŗ' | 'ř' => "r".to_string(),
    'Ś' | 'Ŝ' | 'Ş' | 'Š' | 'ś' | 'ŝ' | 'ş' | 'š' => "s".to_string(),
    'Ţ' | 'Ť' | 'Ŧ' | 'ţ' | 'ť' | 'ŧ' => "t".to_string(),
    'Ù' | 'Ú' | 'Û' | 'Ü' | 'Ũ' | 'Ū' | 'Ŭ' | 'Ů' | 'Ű' | 'Ų' | 'ù' | 'ú' | 'û' | 'ü' | 'ũ'
    | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => "u".to_string(),
    'Ŵ' | 'ŵ' => "w".to_string(),
    'Ý' | 'Ŷ' | 'Ÿ' | 'ý' | 'ÿ' | 'ŷ' => "y".to_string(),
    'Ź' | 'Ż' | 'Ž' | 'ź' | 'ż' | 'ž' => "z".to_string(),
    'Þ' | 'þ' => "th".to_string(),
    _ => ch.to_lowercase().collect(),
  }
}

fn fullwidth_ascii(ch: char) -> Option<char> {
  match ch {
    '０'..='９' => char::from_u32((ch as u32 - '０' as u32) + '0' as u32),
    'Ａ'..='Ｚ' => char::from_u32((ch as u32 - 'Ａ' as u32) + 'A' as u32),
    'ａ'..='ｚ' => char::from_u32((ch as u32 - 'ａ' as u32) + 'a' as u32),
    _ => None,
  }
}
