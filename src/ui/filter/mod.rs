use crate::command::Filter;
use crate::utils::color::parse_color;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

// draw inputbox area
mod input;
// draw book list table
mod table;
// render highlight
mod highlight;

pub struct UiHandler {
    config: crate::config::ui::Filter
}



impl UiHandler {
    pub fn draw(&self,f: &mut Frame, app: &Filter) {
        // set the ui basic layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            // set margin space to 1
            .margin(1)
            // set inputbox's height to 3, book table's takes the rest
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            // get chunks layout
            .split(f.size());

        // draw inputbox
        let input_paragraph = Paragraph::new(app.get_input().as_str())
            // set inputbox style
            .style(Style::default().fg(parse_color(&self.config.inputbox.fg)))
            // set border and title
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(parse_color(&self.config.inputbox.border.fg)))
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Search (Enter to open, Ctrl+C/Esc to quit) ",
                        Style::default().fg(parse_color(&self.config.inputbox.title.fg)),
                    )),
            );
        f.render_widget(input_paragraph, chunks[0]);

        f.set_cursor(chunks[0].x + app.get_input().width() as u16 + 1, chunks[0].y + 1);

        // get layout of table columns
        let mut columns: Vec<_> = self.config.table
            .columns
            .iter()
            .filter(|c| c.ratio > 0)
            .collect();
        columns.sort_by_key(|c| c.position);

        // the label of columns
        let label_cells = columns.iter().map(|c| {
            Cell::from(c.label.clone()).style(
                Style::default()
                    .fg(parse_color(&c.label_fg))
                    .add_modifier(Modifier::BOLD),
            )
        });
        let label = Row::new(label_cells).height(1).bottom_margin(1);

        let hovered_index = app.table_state.selected();

        let rows = app.filtered_books.iter().enumerate().map(|(i, item)| {
            let is_hovered = hovered_index.map_or(false, |s| s == i);
            let cells = columns.iter().map(|col_config| {
                let (fg_color_str, bg_color_str) = if is_hovered {
                    (&col_config.selected_fg, &col_config.selected_bg)
                } else {
                    (&col_config.fg, &col_config.bg)
                };

                let text = if let Some(default_meta) = item.metadata.get("default") {
                    match col_config.name.as_str() {
                        "title" => &default_meta.title,
                        "authors" => &default_meta.authors,
                        "series" => &default_meta.series,
                        "tags" => &default_meta.tags,
                        _ => "",
                    }
                } else {
                    ""
                };

                let line = create_highlighted_line(
                    text,
                    &app.input,
                    &app.pinyin_fuzzy_map,
                    parse_color(fg_color_str),
                    parse_color(&col_config.highlighted_match_fg),
                    app.config.pinyin_search_enabled,
                );

                Cell::from(line).style(Style::default().bg(parse_color(bg_color_str)))
            });
            Row::new(cells).height(1)
        });

        let widths: Vec<_> = columns
            .iter()
            .map(|c| Constraint::Percentage(c.width_ratio))
            .collect();

        let table = Table::new(rows, widths)
            .header(label)
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
    }
    fn create_highlighted_line
}
