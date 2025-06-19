use crate::app::App;
use crate::config::parse_color;
use crate::pinyin::{get_simple_pinyin, to_canonical_pinyin};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Cell, Paragraph, Row, Table},
    Frame,
};
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

pub fn draw(f: &mut Frame, app: &mut App) {
    let colors = &app.config.colors;

    // Removed the status bar from the layout.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let input_paragraph = Paragraph::new(app.input.as_str())
        .style(Style::default().fg(parse_color(&colors.search_box_text)))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(parse_color(&colors.search_box_border_fg)))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " Search (Enter to open, Ctrl+C/Esc to quit) ",
                    Style::default().fg(parse_color(&colors.search_box_title)),
                )),
        );
    f.render_widget(input_paragraph, chunks[0]);

    f.set_cursor(
        chunks[0].x + app.input.width() as u16 + 1,
        chunks[0].y + 1,
    );

    let visible_columns: Vec<_> = app.config.columns.iter().filter(|c| c.width_ratio > 0).collect();

    let header_cells = visible_columns
        .iter()
        .map(|c| Cell::from(c.name.clone()).style(
            Style::default()
                .fg(parse_color(&c.header_fg))
                .add_modifier(Modifier::BOLD),
        ));
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    let selected_index = app.table_state.selected();

    let rows = app.filtered_books.iter().enumerate().map(|(i, item)| {
        let is_selected = selected_index.map_or(false, |s| s == i);
        let cells = visible_columns.iter().map(|col_config| {
            let (fg_color_str, bg_color_str) = if is_selected {
                (&col_config.selected_fg, &col_config.selected_bg)
            } else {
                (&col_config.fg, &col_config.bg)
            };

            let text = match col_config.name.as_str() {
                "title" => &item.title,
                "author" => &item.author,
                "series" => &item.series,
                "tags" => &item.tags,
                _ => "",
            };

            let line = create_highlighted_line(
                text,
                &app.input,
                &app.canonical_map,
                parse_color(fg_color_str),
                parse_color(&col_config.highlighted_match_fg),
            );

            Cell::from(line).style(Style::default().bg(parse_color(bg_color_str)))
        });
        Row::new(cells).height(1)
    });

    let widths: Vec<_> = visible_columns.iter().map(|c| Constraint::Percentage(c.width_ratio)).collect();

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(parse_color(&colors.table_border_fg)))
                .border_type(BorderType::Rounded)
                .title(Span::styled(
                    " Book List (↑/↓/Scroll) ",
                    Style::default().fg(parse_color(&colors.table_title_fg)),
                )),
        )
        .column_spacing(0)
        .highlight_style(Style::default());

    f.render_stateful_widget(table, chunks[1], &mut app.table_state);

    // Removed the status bar widget rendering.
}

fn create_highlighted_line<'a>(
    text: &'a str,
    query: &'a str,
    canonical_map: &HashMap<String, String>,
    base_fg_color: Color,
    highlight_fg_color: Color,
) -> Line<'a> {
    if query.is_empty() {
        return Line::from(Span::styled(text, Style::default().fg(base_fg_color)));
    }

    let base_style = Style::default().fg(base_fg_color);

    let lower_query = query.to_lowercase();

    if let Some((match_byte_start, _)) = text
        .char_indices()
        .find(|(i, _)| text[*i..].to_lowercase().starts_with(&lower_query))
    {
        let query_char_len = lower_query.chars().count();
        let match_byte_end = text[match_byte_start..]
            .char_indices()
            .nth(query_char_len)
            .map(|(i, _)| match_byte_start + i)
            .unwrap_or(text.len());

        return Line::from(vec![
            Span::styled(&text[..match_byte_start], base_style),
            Span::styled(
                &text[match_byte_start..match_byte_end],
                Style::default()
                    .fg(highlight_fg_color)
                    .add_modifier(Modifier::UNDERLINED),
            ),
            Span::styled(&text[match_byte_end..], base_style),
        ]);
    }

    let lower_query_no_space = lower_query.replace(' ', "");
    if lower_query_no_space.is_empty()
        || !lower_query_no_space
            .chars()
            .all(|c| c.is_ascii_alphabetic())
    {
        return Line::from(Span::styled(text, base_style));
    }

    let canonical_query = to_canonical_pinyin(&lower_query_no_space, canonical_map);

    let text_chars: Vec<_> = text.chars().collect();
    for i in 0..text_chars.len() {
        let mut current_canonical_pinyin = String::new();
        for j in i..text_chars.len() {
            let char_as_str = text_chars[j].to_string();
            let pinyin_char = get_simple_pinyin(&char_as_str);
            let canonical_pinyin_char = to_canonical_pinyin(&pinyin_char, canonical_map);
            current_canonical_pinyin.push_str(&canonical_pinyin_char);

            if !canonical_query.starts_with(&current_canonical_pinyin) {
                break;
            }

            if current_canonical_pinyin == canonical_query {
                let prefix: String = text_chars[..i].iter().collect();
                let highlighted: String = text_chars[i..=j].iter().collect();
                let suffix: String = if j + 1 < text_chars.len() {
                    text_chars[j + 1..].iter().collect()
                } else {
                    String::new()
                };

                return Line::from(vec![
                    Span::styled(prefix, base_style),
                    Span::styled(
                        highlighted,
                        Style::default()
                            .fg(highlight_fg_color)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(suffix, base_style),
                ]);
            }
        }
    }

    Line::from(Span::styled(text, base_style))
}
