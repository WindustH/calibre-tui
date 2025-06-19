use pinyin::ToPinyin;
use std::collections::HashMap;

/// 将模糊音组列表预处理为更易于查找的“标准型”哈希表。
/// This function is unchanged.
pub fn build_canonical_map(fuzzy_groups: &[Vec<String>]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for group in fuzzy_groups {
        if let Some(canonical_form) = group.first() {
            for pinyin in group {
                map.insert(pinyin.clone(), canonical_form.clone());
            }
        }
    }
    map
}

/// 将一个拼音字符串转换为其“标准型”。
/// This function is unchanged.
pub fn to_canonical_pinyin(pinyin: &str, canonical_map: &HashMap<String, String>) -> String {
    if canonical_map.is_empty() {
        return pinyin.to_string();
    }

    let mut sorted_keys: Vec<_> = canonical_map.keys().collect();
    sorted_keys.sort_by(|a, b| b.len().cmp(&a.len()));

    let mut result = String::new();
    let mut i = 0;
    while i < pinyin.len() {
        let remaining_pinyin = &pinyin[i..];
        let mut found_match = false;

        for key in &sorted_keys {
            if remaining_pinyin.starts_with(key.as_str()) {
                if let Some(canonical) = canonical_map.get(*key) {
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
    result
}

/// 为给定的文本字符串生成一个简单的拼音字符串，不支持多音字。
/// e.g. "重庆" -> "chongqing"
pub fn get_simple_pinyin(s: &str) -> String {
    s.chars()
        .map(|c| {
            // For each character, get its single pinyin, or the character itself if no pinyin exists.
            match c.to_pinyin() {
                Some(pinyin) => pinyin.plain().to_string(),
                None => c.to_string(),
            }
        })
        .collect::<String>()
}
