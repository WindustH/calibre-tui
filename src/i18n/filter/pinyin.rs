use super::{IndexedText, Translator, normalize_plain_query};
use crate::config::FilterConfig;
use anyhow::{Result, anyhow};
use pinyin::ToPinyin;
use std::collections::HashMap;

pub struct PinyinFilter {
  fuzzy_map: Option<HashMap<String, String>>,
}

impl PinyinFilter {
  pub fn new(config: &FilterConfig) -> Self {
    Self {
      fuzzy_map: config
        .pinyin_fuzzy
        .then(|| build_fuzzy_map(&config.pinyin_fuzzy_groups)),
    }
  }

  fn apply_fuzzy_to_indexed_text(&self, indexed: &IndexedText) -> Result<IndexedText> {
    let Some(fuzzy_map) = &self.fuzzy_map else {
      return Ok(indexed.clone());
    };
    if fuzzy_map.is_empty() {
      return Ok(indexed.clone());
    }

    let syllables = indexed
      .token_bounds
      .windows(2)
      .map(|bounds| {
        let syllable = indexed
          .text
          .chars()
          .skip(bounds[0])
          .take(bounds[1] - bounds[0])
          .collect::<String>();
        self.apply_fuzzy_to_query(&syllable)
      })
      .collect::<Result<Vec<_>>>()?;

    let mut text = String::new();
    let mut token_bounds = vec![0];
    for syllable in syllables {
      text.push_str(&syllable);
      token_bounds.push(text.chars().count());
    }

    Ok(IndexedText { text, token_bounds })
  }

  fn apply_fuzzy_to_query(&self, query: &str) -> Result<String> {
    let Some(fuzzy_map) = &self.fuzzy_map else {
      return Ok(query.to_string());
    };
    if fuzzy_map.is_empty() {
      return Ok(query.to_string());
    }

    let mut keys = fuzzy_map.keys().collect::<Vec<_>>();
    keys.sort_by_key(|key| std::cmp::Reverse(key.len()));

    let mut result = String::new();
    let mut byte_index = 0;
    while byte_index < query.len() {
      let rest = &query[byte_index..];
      let mut matched = false;

      for key in &keys {
        if rest.starts_with(key.as_str()) {
          let canonical = fuzzy_map
            .get(*key)
            .ok_or_else(|| anyhow!("missing fuzzy pinyin key '{}'", key))?;
          result.push_str(canonical);
          byte_index += key.len();
          matched = true;
          break;
        }
      }

      if !matched {
        let Some(ch) = rest.chars().next() else {
          break;
        };
        result.push(ch);
        byte_index += ch.len_utf8();
      }
    }

    Ok(result)
  }
}

impl Translator for PinyinFilter {
  fn index_text(&self, text: &str) -> Result<IndexedText> {
    let mut indexed = String::new();
    let mut token_bounds = vec![0];

    for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
      indexed.push_str(&char_to_search_text(ch));
      token_bounds.push(indexed.chars().count());
    }

    self.apply_fuzzy_to_indexed_text(&IndexedText {
      text: indexed,
      token_bounds,
    })
  }

  fn normalize_query(&self, query: &str) -> Result<String> {
    self.apply_fuzzy_to_query(&normalize_plain_query(query))
  }
}

fn build_fuzzy_map(groups: &[Vec<String>]) -> HashMap<String, String> {
  let mut map = HashMap::new();
  for group in groups {
    if let Some(canonical) = group.first() {
      for value in group {
        map.insert(value.clone(), canonical.clone());
      }
    }
  }
  map
}

fn char_to_search_text(ch: char) -> String {
  if let Some(pinyin) = ch.to_pinyin() {
    return pinyin.plain().to_lowercase();
  }

  match ch {
    '，' => ",".to_string(),
    '《' => "<".to_string(),
    '》' => ">".to_string(),
    '：' => ":".to_string(),
    '；' => ";".to_string(),
    '—' => "-".to_string(),
    '“' | '”' => "\"".to_string(),
    '‘' | '’' => "'".to_string(),
    '（' => "(".to_string(),
    '）' => ")".to_string(),
    '【' => "[".to_string(),
    '】' => "]".to_string(),
    '！' => "!".to_string(),
    '？' => "?".to_string(),
    '。' => ".".to_string(),
    '、' => ",".to_string(),
    '０'..='９' => char::from_u32((ch as u32 - '０' as u32) + '0' as u32)
      .unwrap_or(ch)
      .to_string(),
    'Ａ'..='Ｚ' => char::from_u32((ch as u32 - 'Ａ' as u32) + 'a' as u32)
      .unwrap_or(ch)
      .to_string(),
    'ａ'..='ｚ' => char::from_u32((ch as u32 - 'ａ' as u32) + 'a' as u32)
      .unwrap_or(ch)
      .to_string(),
    _ => ch.to_lowercase().collect(),
  }
}
