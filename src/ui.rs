use crate::filter::{BookHighlights, HighlightRanges, SearchResult};
use crate::utils::book::Book;
use ratatui::layout::Rect;
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::collections::BTreeSet;
use unicode_width::UnicodeWidthStr;

struct Column {
  key: ColumnKey,
  label: &'static str,
  width: u16,
  fg: Color,
}

#[derive(Clone, Copy)]
enum ColumnKey {
  Title,
  Authors,
  Series,
  Tags,
}

const COLUMNS: [Column; 4] = [
  Column {
    key: ColumnKey::Title,
    label: "title",
    width: 40,
    fg: Color::White,
  },
  Column {
    key: ColumnKey::Authors,
    label: "authors",
    width: 20,
    fg: Color::Cyan,
  },
  Column {
    key: ColumnKey::Series,
    label: "series",
    width: 20,
    fg: Color::White,
  },
  Column {
    key: ColumnKey::Tags,
    label: "tags",
    width: 20,
    fg: Color::Cyan,
  },
];

pub struct DrawState<'a> {
  pub input: &'a str,
  pub books: &'a [Book],
  pub results: &'a [SearchResult],
  pub table_state: &'a mut TableState,
  pub selected_book_indices: &'a BTreeSet<usize>,
  pub print_path: bool,
}

pub fn draw(frame: &mut Frame, area: Rect, state: DrawState<'_>) {
  let DrawState {
    input,
    books,
    results,
    table_state,
    selected_book_indices,
    print_path,
  } = state;

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints([Constraint::Length(3), Constraint::Min(0)])
    .split(area);

  draw_input(
    frame,
    chunks[0],
    input,
    selected_book_indices.len(),
    print_path,
  );
  draw_table(
    frame,
    chunks[1],
    books,
    results,
    table_state,
    selected_book_indices,
  );
}

fn draw_input(frame: &mut Frame, area: Rect, input: &str, selected_count: usize, print_path: bool) {
  let action = if print_path { "print path" } else { "open" };
  let title = format!(" Search ({action}) [{selected_count} selected] ");
  let input_box = Paragraph::new(input)
    .style(Style::default().fg(Color::White))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .border_type(BorderType::Rounded)
        .title(Span::styled(title, Style::default().fg(Color::Blue))),
    );
  frame.render_widget(input_box, area);

  let max_cursor_width = area.width.saturating_sub(2) as usize;
  let cursor_width = input.width().min(max_cursor_width);
  frame.set_cursor_position((area.x + cursor_width as u16 + 1, area.y + 1));
}

fn draw_table(
  frame: &mut Frame,
  area: Rect,
  books: &[Book],
  results: &[SearchResult],
  table_state: &mut TableState,
  selected_book_indices: &BTreeSet<usize>,
) {
  let header = Row::new(COLUMNS.iter().map(|column| {
    Cell::from(column.label).style(
      Style::default()
        .fg(Color::Blue)
        .add_modifier(Modifier::BOLD),
    )
  }))
  .height(1)
  .bottom_margin(1);

  let selected = table_state.selected();
  let rows = results.iter().enumerate().map(|(row_index, result)| {
    let book = &books[result.book_index];
    let is_hovered = selected == Some(row_index);
    let is_marked = selected_book_indices.contains(&result.book_index);

    Row::new(COLUMNS.iter().map(|column| {
      let (text, ranges) = field_text_and_highlights(book, &result.highlights, column.key);
      let base_style = if is_marked && is_hovered {
        Style::default().fg(Color::Black).bg(Color::Yellow)
      } else if is_marked {
        Style::default().fg(Color::Black).bg(Color::Green)
      } else if is_hovered {
        Style::default().fg(Color::Black).bg(Color::Blue)
      } else {
        Style::default().fg(column.fg)
      };
      let highlight_style = if is_marked && is_hovered {
        Style::default()
          .fg(Color::Red)
          .bg(Color::Yellow)
          .add_modifier(Modifier::BOLD)
      } else if is_marked {
        Style::default()
          .fg(Color::Yellow)
          .bg(Color::Green)
          .add_modifier(Modifier::BOLD)
      } else if is_hovered {
        Style::default()
          .fg(Color::Yellow)
          .bg(Color::Blue)
          .add_modifier(Modifier::BOLD)
      } else {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
      };

      Cell::from(highlighted_line(&text, ranges, base_style, highlight_style)).style(base_style)
    }))
    .height(1)
  });

  let widths = COLUMNS
    .iter()
    .map(|column| Constraint::Percentage(column.width))
    .collect::<Vec<_>>();

  let table = Table::new(rows, widths)
    .header(header)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .border_type(BorderType::Rounded)
        .title(Span::styled(
          " Book List ",
          Style::default().fg(Color::Blue),
        )),
    )
    .column_spacing(0)
    .row_highlight_style(Style::default());

  frame.render_stateful_widget(table, area, table_state);
}

fn field_text_and_highlights<'a>(
  book: &'a Book,
  highlights: &'a BookHighlights,
  key: ColumnKey,
) -> (String, &'a HighlightRanges) {
  match key {
    ColumnKey::Title => (book.title.clone(), &highlights.title),
    ColumnKey::Authors => (book.authors.join(" & "), &highlights.authors),
    ColumnKey::Series => (book.series.clone(), &highlights.series),
    ColumnKey::Tags => (book.tags.join(", "), &highlights.tags),
  }
}

fn highlighted_line(
  text: &str,
  ranges: &HighlightRanges,
  base_style: Style,
  highlight_style: Style,
) -> Line<'static> {
  if ranges.is_empty() {
    return Line::from(Span::styled(text.to_string(), base_style));
  }

  let mut sorted_ranges = ranges.clone();
  sorted_ranges.sort_unstable_by_key(|range| range.0);
  let mut ranges = sorted_ranges.iter().peekable();
  let mut current_range = ranges.next();
  let mut non_space_index = 0;
  let mut current_text = String::new();
  let mut spans = Vec::new();

  for ch in text.chars() {
    if current_range.is_some_and(|range| non_space_index == range.0) && !current_text.is_empty() {
      spans.push(Span::styled(std::mem::take(&mut current_text), base_style));
    }

    current_text.push(ch);

    if !ch.is_whitespace() {
      non_space_index += 1;
    }

    if current_range.is_some_and(|range| non_space_index == range.1) {
      spans.push(Span::styled(
        std::mem::take(&mut current_text),
        highlight_style,
      ));
      current_range = ranges.next();
    }
  }

  if !current_text.is_empty() {
    spans.push(Span::styled(current_text, base_style));
  }

  Line::from(spans)
}
