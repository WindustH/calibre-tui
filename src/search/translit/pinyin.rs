use super::{IndexedText, Translator, fullwidth_ascii, normalize_plain_query};
use crate::config::FilterConfig;
use pinyin::ToPinyin;
use std::borrow::Cow;
use std::collections::BTreeMap;

pub(super) struct ChinesePinyinTranslator {
  /// `(fragment, canonical)` pairs, longest fragment first. Empty when fuzzy
  /// matching is disabled.
  fuzzy_rules: Vec<(String, String)>,
}

impl ChinesePinyinTranslator {
  pub(super) fn new(config: &FilterConfig) -> Self {
    let fuzzy_rules = if config.pinyin_fuzzy {
      fuzzy_rules(&config.pinyin_fuzzy_groups)
    } else {
      Vec::new()
    };
    Self { fuzzy_rules }
  }

  /// Rewrite every fuzzy fragment to its canonical form, scanning left to
  /// right and preferring the longest fragment at each position. Allocates
  /// only when a rule applies.
  fn apply_fuzzy<'a>(&self, text: &'a str) -> Cow<'a, str> {
    let mut rewritten: Option<String> = None;
    let mut rest = text;
    while let Some(ch) = rest.chars().next() {
      let rule = self
        .fuzzy_rules
        .iter()
        .find(|(fragment, _)| rest.starts_with(fragment.as_str()));
      let consumed = match rule {
        Some((fragment, canonical)) => {
          let done = text.len() - rest.len();
          rewritten
            .get_or_insert_with(|| text[..done].to_string())
            .push_str(canonical);
          fragment.len()
        }
        None => {
          if let Some(rewritten) = &mut rewritten {
            rewritten.push(ch);
          }
          ch.len_utf8()
        }
      };
      rest = &rest[consumed..];
    }
    rewritten.map_or(Cow::Borrowed(text), Cow::Owned)
  }
}

impl Translator for ChinesePinyinTranslator {
  /// One token per source char; fuzzy rules apply within each syllable.
  fn index_text(&self, text: &str) -> IndexedText {
    let mut indexed = IndexedText::default();
    let mut syllable = String::new();
    for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
      syllable.clear();
      push_char_search_text(ch, &mut syllable);
      indexed.push_token(&self.apply_fuzzy(&syllable));
    }
    indexed
  }

  fn normalize_query(&self, query: &str) -> String {
    self.apply_fuzzy(&normalize_plain_query(query)).into_owned()
  }
}

/// Map every group member to the group's first entry. Empty fragments are
/// skipped (they would match everywhere); later groups win on conflicts.
fn fuzzy_rules(groups: &[Vec<String>]) -> Vec<(String, String)> {
  let mut map = BTreeMap::new();
  for group in groups {
    let Some(canonical) = group.first() else {
      continue;
    };
    let canonical = normalize_plain_query(canonical);
    for value in group {
      let fragment = normalize_plain_query(value);
      if !fragment.is_empty() {
        map.insert(fragment, canonical.clone());
      }
    }
  }

  let mut rules = map.into_iter().collect::<Vec<_>>();
  rules.sort_by_key(|(fragment, _)| std::cmp::Reverse(fragment.len()));
  rules
}

/// Toneless pinyin for Hanzi; full-width ASCII and CJK punctuation as ASCII;
/// anything else lowercased.
fn push_char_search_text(ch: char, out: &mut String) {
  if let Some(pinyin) = ch.to_pinyin() {
    out.extend(pinyin.plain().chars().flat_map(char::to_lowercase));
    return;
  }
  if let Some(ascii) = fullwidth_ascii(ch) {
    out.push(ascii.to_ascii_lowercase());
    return;
  }

  let ascii = match ch {
    '，' | '、' => ",",
    '《' => "<",
    '》' => ">",
    '：' => ":",
    '；' => ";",
    '—' => "-",
    '“' | '”' => "\"",
    '‘' | '’' => "'",
    '（' => "(",
    '）' => ")",
    '【' => "[",
    '】' => "]",
    '！' => "!",
    '？' => "?",
    '。' => ".",
    _ => {
      out.extend(ch.to_lowercase());
      return;
    }
  };
  out.push_str(ascii);
}

#[cfg(test)]
mod tests {
  use super::*;

  fn translator(groups: &[&[&str]]) -> ChinesePinyinTranslator {
    ChinesePinyinTranslator::new(&FilterConfig {
      pinyin_fuzzy: true,
      pinyin_fuzzy_groups: groups
        .iter()
        .map(|group| group.iter().map(|value| value.to_string()).collect())
        .collect(),
      ..FilterConfig::default()
    })
  }

  #[test]
  fn fuzzy_groups_canonicalize_index_and_query() {
    let translator = translator(&[&["on", "ong"], &["an", "ang"]]);
    let indexed = translator.index_text("中国 上海");
    assert_eq!(indexed.text(), "zhonguoshanhai");
    assert_eq!(translator.normalize_query("ZhongGuo"), "zhonguo");
    assert_eq!(
      indexed.find(&translator.normalize_query("shanghai")),
      Some((2, 4))
    );
  }

  #[test]
  fn empty_fuzzy_fragment_does_not_hang() {
    let translator = translator(&[&["", "x"], &["an", ""], &[]]);
    assert_eq!(translator.normalize_query("xan"), "an");
    assert_eq!(translator.index_text("A").text(), "a");
  }

  #[test]
  fn fullwidth_ascii_is_lowercased() {
    let translator = translator(&[]);
    assert_eq!(translator.index_text("Ｆ１").text(), "f1");
  }
}
