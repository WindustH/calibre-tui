use crate::widget::filter::BooksHighlights;
use crate::utils::book::{Books, Uuids};
use crate::utils::color::parse_color;
use anyhow::Result;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};
use unicode_width::UnicodeWidthStr;

pub struct Handler {
    config: crate::config::ui::Filter,
}

impl Handler {
    pub fn new(config: &crate::config::ui::Filter) -> Result<Self> {
        Ok(Self {
            config: config.clone(),
        })
    }
    pub fn draw(
        &self,
        frame: &mut Frame,                  // frame to draw
        input: &str,                     // input in filter inputbox
        filtered_uuids: &Uuids,             // uuids of filtered books
        books_highlights: &BooksHighlights, // highlights
        database: &Books,                   // books data
        table_state: &mut TableState,       // table state
    ) {
        // set the ui basic layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            // set margin space to 1
            .margin(1)
            // set inputbox's height to 3, book table's takes the rest
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            // get chunks layout
            .split(frame.size());

        // draw inputbox
        let input_paragraph = Paragraph::new(input)
            // set inputbox style
            .style(Style::default().fg(parse_color(&self.config.inputbox.fg)))
            // set border and title
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(parse_color(&self.config.inputbox.border.fg)))
                    // rounded rect
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Search (Enter to open, Ctrl+C/Esc to quit) ",
                        Style::default().fg(parse_color(&self.config.inputbox.title.fg)),
                    )),
            );
        frame.render_widget(input_paragraph, chunks[0]);

        // make cursor visible
        frame.set_cursor(chunks[0].x + input.width() as u16 + 1, chunks[0].y + 1);

        // get layout of table columns
        let mut columns: Vec<_> = self
            .config
            .table
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

        // get the hoverd tab index
        let hovered_index = table_state.selected();

        let rows = filtered_uuids.iter().enumerate().map(|(i, uuid)| {
            let is_hovered = hovered_index.map_or(false, |s| s == i);
            let cells = columns.iter().map(|col_config| {
                let (fg_color_str, bg_color_str) = if is_hovered {
                    (&col_config.hovered_fg, &col_config.hovered_bg)
                } else {
                    (&col_config.fg, &"Reset".to_string())
                };

                let text = if let Some(book_dat) = database.get(uuid) {
                    match col_config.label.as_str() {
                        "title" => book_dat.title.to_string(),
                        "authors" => book_dat.authors.join(" & "),
                        "series" => book_dat.series.to_string(),
                        "tags" => book_dat.tags.join(", "),
                        _ => "".to_string(),
                    }
                } else {
                    "".to_string()
                };

                let line = {
                    // get highlights by uuid
                    if let Some(book_highlights) = books_highlights.get(uuid) {
                        // get highlights based on label
                        let highlights = match col_config.label.as_str() {
                            "title" => &book_highlights.title,
                            "authors" => &book_highlights.authors,
                            "series" => &book_highlights.series,
                            "tags" => &book_highlights.tags,
                            // fallback as empty array
                            _ => &vec![],
                        };

                        // if highlights is empty, or only contains a "no match" marker
                        // render the entire text with the default style
                        if highlights.is_empty() || (highlights.len() == 1 && !highlights[0].0) {
                            Line::from(Span::styled(
                                text,
                                Style::default().fg(parse_color(fg_color_str)),
                            ))
                        } else {
                            // construct the line with highlighted and non-highlighted parts
                            let mut spans = vec![];
                            let mut non_space_idx = 0; // tracks position in the text without spaces

                            // clone and sort highlights by their start position
                            let mut sorted_highlights = highlights.to_vec();
                            sorted_highlights.sort_unstable_by_key(|h| h.1);

                            let mut highlight_iter = sorted_highlights.iter().peekable();
                            let mut current_highlight = highlight_iter.next();

                            let mut current_span_text = String::new();

                            for (_char_idx, ch) in text.chars().enumerate() {
                                let is_space = ch.is_whitespace();

                                // check if the current character is the start of a highlight
                                if let Some(&(_, start, _)) = current_highlight {
                                    if non_space_idx == start {
                                        // push the preceding non-highlighted text if it exists
                                        if !current_span_text.is_empty() {
                                            spans.push(Span::styled(
                                                std::mem::take(&mut current_span_text),
                                                Style::default().fg(parse_color(fg_color_str)),
                                            ));
                                        }
                                    }
                                }

                                current_span_text.push(ch);

                                if !is_space {
                                    non_space_idx += 1;
                                }

                                // check if the current character is the end of a highlight
                                if let Some(&(_, _, end)) = current_highlight {
                                    if non_space_idx == end {
                                        // push the highlighted text
                                        let highlighted_fg_color = if is_hovered {
                                            parse_color(&col_config.hovered_highlighted_fg)
                                        } else {
                                            parse_color(&col_config.highlighted_fg)
                                        };
                                        spans.push(Span::styled(
                                            std::mem::take(&mut current_span_text),
                                            Style::default()
                                                .fg(highlighted_fg_color)
                                                .add_modifier(Modifier::BOLD),
                                        ));
                                        current_highlight = highlight_iter.next();
                                    }
                                }
                            }

                            // add the remaining text after the last highlight, if any.
                            if !current_span_text.is_empty() {
                                spans.push(Span::styled(
                                    current_span_text,
                                    Style::default().fg(parse_color(fg_color_str)),
                                ));
                            }

                            Line::from(spans)
                        }
                    } else {
                        // no highlight information, render text with default style
                        Line::from(Span::styled(
                            text,
                            Style::default().fg(parse_color(fg_color_str)),
                        ))
                    }
                };

                Cell::from(line).style(Style::default().bg(parse_color(bg_color_str)))
            });
            Row::new(cells).height(1)
        });

        let widths: Vec<_> = columns
            .iter()
            .map(|c| Constraint::Percentage(c.ratio))
            .collect();

        let table = Table::new(rows, widths)
            .header(label)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(parse_color(&self.config.table.border.fg)))
                    .border_type(BorderType::Rounded)
                    .title(Span::styled(
                        " Book List (↑/↓/Scroll) ",
                        Style::default().fg(parse_color(&self.config.table.title.fg)),
                    )),
            )
            .column_spacing(0)
            .highlight_style(Style::default());

        frame.render_stateful_widget(table, chunks[1], table_state);
    }
}
