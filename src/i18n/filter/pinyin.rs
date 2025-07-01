// use crate::utils::book::Metadata;
use super::{IR, TString};
use anyhow::{Result,anyhow};
use pinyin::ToPinyin;
use std::collections::HashMap;

// no need for clone anymore
// #[derive(Clone)]
pub struct Pinyin {
    enabled: bool,
    fuzzy_enabled: bool,
    fuzzy_map: Option<HashMap<String, String>>,
}

impl Pinyin {
    pub fn new(config: &crate::config::i18n::filter::Pinyin) -> Result<Self> {
        if config.enabled {
            Ok(Self {
                enabled: true,
                fuzzy_enabled: config.fuzzy_enabled,
                fuzzy_map: if config.fuzzy_enabled {
                    Some(Self::build_fuzzy_map(&config.fuzzy_groups))
                } else {
                    None
                },
            })
        } else {
            Ok(Self {
                enabled: false,
                fuzzy_enabled: false,
                fuzzy_map: None,
            })
        }
    }
    fn build_fuzzy_map(fuzzy_groups: &[Vec<String>]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for group in fuzzy_groups {
            if let Some(uni_form) = group.first() {
                for pinyin in group {
                    map.insert(pinyin.clone(), uni_form.clone());
                }
            }
        }
        map
    }
    fn apply_fuzzy_map_to_translation(&self, translation: &TString) -> Result<TString> {
        if !self.enabled {
            return Err(anyhow!("pinyin is disabled"));
        }
        if !self.fuzzy_enabled {
            return Err(anyhow!("pinyin fuzzy map is disabled"));
        }

        if let Some(fuzzy_map) = self.fuzzy_map.as_ref() {
            // fuzzy map is empty
            if fuzzy_map.is_empty() {
                return Ok(translation.clone());
            }

            // sort the key. longer pinyin has higher priority
            let mut sorted_keys: Vec<_> = fuzzy_map.keys().collect();
            sorted_keys.sort_by(|a, b| b.len().cmp(&a.len()));

            // collect new syllabels into a vec of string
            let new_syllables: Vec<String> = translation
                .1
                .windows(2) // iterate through the nearby pairs of index
                .map(|indices| {
                    let word = &translation.0[indices[0]..indices[1]];

                    // find the first match in fuzzy map
                    sorted_keys
                        .iter()
                        .find_map(|key| {
                            let replaced = word.replace(key.as_str(), &fuzzy_map[*key]);
                            if replaced != word {
                                Some(replaced)
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| word.to_string())
                })
                .collect();

            // build new translation based on new syllables
            let mut result_string = String::new();
            let mut result_indices = vec![0];

            for syllable in &new_syllables {
                result_string.push_str(syllable);
                result_indices.push(result_string.len());
            }

            Ok((result_string, result_indices))
        } else {
            Err(anyhow!("fuzzy map is not correctly initialized"))
        }
    }
    fn apply_fuzzy_map_to_input(&self, pinyin: &str) -> Result<String> {
        if !self.enabled {
            return Err(anyhow!("pinyin is disabled"));
        }
        if !self.fuzzy_enabled {
            return Err(anyhow!("pinyin fuzzy map is disabled"));
        }
        if let Some(fuzzy_map) = &self.fuzzy_map {
            if fuzzy_map.is_empty() {
                return Ok(pinyin.to_string());
            }

            let mut sorted_keys: Vec<_> = fuzzy_map.keys().collect();
            sorted_keys.sort_by(|a, b| b.len().cmp(&a.len()));

            let mut result = String::new();
            let mut i = 0;
            while i < pinyin.len() {
                let remaining_pinyin = &pinyin[i..];
                let mut found_match = false;

                for key in &sorted_keys {
                    if remaining_pinyin.starts_with(key.as_str()) {
                        if let Some(canonical) = fuzzy_map.get(*key) {
                            result.push_str(canonical);
                            i += key.len();
                            found_match = true;
                            break;
                        }
                    }
                }

                if !found_match {
                    let ch = remaining_pinyin.chars().next().unwrap();
                    result.push(ch);
                    i += ch.len_utf8();
                }
            }
            Ok(result)
        } else {
            Err(anyhow!("fuzzy map is not correctly initialized"))
        }
    }

    pub fn get_translation(&self, s: &str) -> Result<TString> {
        if !self.enabled {
            return Err(anyhow!("pinyin is disabled"));
        };

        let pinyin_parts: Vec<String> = s
            .chars()
            .map(|c| match c.to_pinyin() {
                Some(pinyin) => pinyin.plain().to_string(),
                None => c.to_string(),
            })
            .collect();

        let mut combined_string = String::new();
        let mut indices = vec![0];

        for part in &pinyin_parts {
            combined_string.push_str(part);
            indices.push(combined_string.len());
        }

        let original_translation = (combined_string, indices);
        if self.fuzzy_enabled {
            Ok(self.apply_fuzzy_map_to_translation(&original_translation)?)
        } else {
            Ok(original_translation)
        }
    }
}

impl IR for Pinyin {
    fn trans_book_info(&self, s: &str) -> Result<TString> {
        self.get_translation(&s)
    }

    fn trans_input(&self, s: &str) -> Result<String> {
        self.apply_fuzzy_map_to_input(&s)
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}
