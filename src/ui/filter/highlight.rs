use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span}
};
use std::collections::HashMap;

fn create_highlighted_line<'a>(
    text: &'a str,
    query: &'a str,
    pinyin_fuzzy_map: &HashMap<String, String>,
    base_fg_color: Color,
    highlight_fg_color: Color,
    pinyin_search_enabled: bool,
) -> Line<'a> {
    if query.is_empty() {
        return Line::from(Span::styled(text, Style::default().fg(base_fg_color)));
    }

    let base_style = Style::default().fg(base_fg_color);
    let highlight_style = Style::default()
        .fg(highlight_fg_color)
        .add_modifier(Modifier::UNDERLINED);
    let lower_query = query.to_lowercase();

    // 1. Direct Substring Match
    if let Some((match_byte_start, _)) = text
        .char_indices()
        .find(|(i, _)| text[*i..].to_lowercase().starts_with(&lower_query))
    {
        let query_char_len = query.chars().count();
        let match_byte_end = text[match_byte_start..]
            .char_indices()
            .nth(query_char_len)
            .map(|(i, _)| match_byte_start + i)
            .unwrap_or(text.len());

        return Line::from(vec![
            Span::styled(&text[..match_byte_start], base_style),
            Span::styled(&text[match_byte_start..match_byte_end], highlight_style),
            Span::styled(&text[match_byte_end..], base_style),
        ]);
    }

    // 2. Pinyin Match (if enabled)
    if !pinyin_search_enabled {
        return Line::from(Span::styled(text, base_style));
    }

    let lower_query_no_space = lower_query.replace(' ', "");
    if lower_query_no_space.is_empty() {
        return Line::from(Span::styled(text, base_style));
    }

    // FIX: Allow numbers in pinyin search query
    let is_pinyin_searchable = lower_query_no_space
        .chars()
        .all(|c| c.is_ascii_alphanumeric());

    if !is_pinyin_searchable {
        return Line::from(Span::styled(text, base_style));
    }

    let pinyin_query = apply_fuzzy_map(&lower_query_no_space, pinyin_fuzzy_map);

    // Generate pinyin for each character in the text, converting to lowercase.
    // Non-Chinese characters (like numbers/letters) will be used as-is.
    let text_pinyins_canonical: Vec<String> = text
        .chars()
        .map(|c| {
            let pinyin = get_pinyin(&c.to_string());
            apply_fuzzy_map(&pinyin, pinyin_fuzzy_map).to_lowercase()
        })
        .collect();

    let combined_pinyin = text_pinyins_canonical.join("");

    // FIX: Use `find` for partial pinyin matches and map back to characters.
    // This makes highlighting consistent with the `contains` logic in the search filter.
    if let Some(match_start_pinyin_idx) = combined_pinyin.find(&pinyin_query) {
        let mut pinyin_len_so_far = 0;
        let mut start_char_idx = 0;
        let mut end_char_idx = 0;

        // Find the character index where the match starts
        for (i, pinyin) in text_pinyins_canonical.iter().enumerate() {
            if pinyin_len_so_far + pinyin.len() > match_start_pinyin_idx {
                start_char_idx = i;
                break;
            }
            pinyin_len_so_far += pinyin.len();
        }

        let match_end_pinyin_idx = match_start_pinyin_idx + pinyin_query.len();
        pinyin_len_so_far = 0;

        // Find the character index where the match ends
        let mut found_end = false;
        for (i, pinyin) in text_pinyins_canonical.iter().enumerate() {
            pinyin_len_so_far += pinyin.len();
            if pinyin_len_so_far >= match_end_pinyin_idx {
                end_char_idx = i;
                found_end = true;
                break;
            }
        }
        if !found_end {
            end_char_idx = text_pinyins_canonical.len() - 1;
        }

        let text_chars: Vec<_> = text.chars().collect();
        let prefix: String = text_chars[..start_char_idx].iter().collect();
        let highlighted: String = text_chars[start_char_idx..=end_char_idx].iter().collect();
        let suffix: String = text_chars
            .get(end_char_idx + 1..)
            .map_or(String::new(), |s| s.iter().collect());

        return Line::from(vec![
            Span::styled(prefix, base_style),
            Span::styled(highlighted, highlight_style),
            Span::styled(suffix, base_style),
        ]);
    }

    Line::from(Span::styled(text, base_style))
}
