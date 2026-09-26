//! The book table. Only the rows that fit on screen are built, so drawing
//! cost does not grow with the number of results.

use crate::layout::Layout;
use crate::library::{Book, BookField};
use crate::search::{SearchResult, TokenRange};
use crate::theme::Theme;
use ratatui::{
  Frame,
  layout::{Constraint, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};
use std::collections::BTreeSet;

pub struct BookTable<'a> {
  pub books: &'a [Book],
  pub results: &'a [SearchResult],
  pub marked: &'a BTreeSet<usize>,
  pub focus: Option<usize>,
  /// First visible row from the previous frame.
  pub scroll: usize,
}

/// The rows shown by the last draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TableViewport {
  pub first_row: usize,
  pub rows: usize,
}

/// Base and match-highlight styles for one row state.
#[derive(Clone, Copy)]
struct RowStyle {
  base: Style,
  highlight: Style,
}

pub(super) fn draw_table(
  frame: &mut Frame,
  area: Rect,
  table: &BookTable<'_>,
  layout: &Layout,
  theme: &Theme,
) -> TableViewport {
  let columns = layout.visible_columns().collect::<Vec<_>>();
  // Two border rows, the header row, and the margin below it.
  let rows = usize::from(area.height.saturating_sub(4));
  let first_row = first_visible_row(table.results.len(), table.focus, table.scroll, rows);
  let visible = &table.results[first_row..table.results.len().min(first_row + rows)];

  let header_style = Style::default()
    .fg(theme.color(&theme.table.header))
    .bg(theme.color(&theme.background))
    .add_modifier(Modifier::BOLD);
  let header = Row::new(
    columns
      .iter()
      .map(|column| Cell::from(column.label.as_str()).style(header_style)),
  )
  .height(1)
  .bottom_margin(1);

  let styles = RowStyles::new(theme);
  let normal_styles = columns
    .iter()
    .map(|column| RowStyle {
      base: styles.normal.base.fg(field_color(column.field, theme)),
      ..styles.normal
    })
    .collect::<Vec<_>>();

  let table_rows = visible.iter().enumerate().map(|(offset, result)| {
    let book = &table.books[result.book_index];
    let hovered = table.focus == Some(first_row + offset);
    let marked = table.marked.contains(&result.book_index);
    Row::new(columns.iter().zip(&normal_styles).map(|(column, &normal)| {
      let style = styles.for_state(marked, hovered).unwrap_or(normal);
      let text = book.field_text(column.field);
      let ranges = result.highlights.ranges(column.field);
      Cell::from(highlighted_line(&text, ranges, style)).style(style.base)
    }))
    .height(1)
  });

  let total_width = columns
    .iter()
    .map(|column| u32::from(column.width))
    .sum::<u32>();
  let widths = columns
    .iter()
    .map(|column| Constraint::Ratio(u32::from(column.width), total_width))
    .collect::<Vec<_>>();

  let widget = Table::new(table_rows, widths)
    .header(header)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.color(&theme.table.border)))
        .border_type(BorderType::Rounded)
        .title(Span::styled(
          " Book List ",
          Style::default().fg(theme.color(&theme.table.title)),
        )),
    )
    .column_spacing(0)
    .row_highlight_style(Style::default());

  let selected = table
    .focus
    .filter(|_| !visible.is_empty())
    .map(|focus| focus - first_row);
  let mut state = TableState::default().with_selected(selected);
  frame.render_stateful_widget(widget, area, &mut state);

  TableViewport { first_row, rows }
}

/// First row to show so the focused row stays visible, scrolling as little as
/// possible from `scroll` (matches ratatui's own table scrolling).
fn first_visible_row(len: usize, focus: Option<usize>, scroll: usize, rows: usize) -> usize {
  let Some(last) = len.checked_sub(1) else {
    return 0;
  };
  let mut first = scroll.min(last);
  if let Some(focus) = focus.map(|focus| focus.min(last)) {
    first = first.min(focus);
    if rows > 0 && focus >= first + rows {
      first = focus + 1 - rows;
    }
  }
  first
}

struct RowStyles {
  normal: RowStyle,
  hovered: RowStyle,
  marked: RowStyle,
  marked_hovered: RowStyle,
}

impl RowStyles {
  fn new(theme: &Theme) -> Self {
    let row = &theme.row;
    let highlight = &theme.highlight;
    let style = |fg: &str, bg: &str, highlight_fg: &str| RowStyle {
      base: Style::default().fg(theme.color(fg)).bg(theme.color(bg)),
      highlight: Style::default()
        .fg(theme.color(highlight_fg))
        .bg(theme.color(bg))
        .add_modifier(Modifier::BOLD),
    };
    Self {
      // The normal foreground is replaced per column by the field color.
      normal: style(&theme.foreground, &theme.background, &highlight.normal),
      hovered: style(
        &row.hover_foreground,
        &row.hover_background,
        &highlight.hover,
      ),
      marked: style(
        &row.selected_foreground,
        &row.selected_background,
        &highlight.selected,
      ),
      marked_hovered: style(
        &row.selected_hover_foreground,
        &row.selected_hover_background,
        &highlight.selected_hover,
      ),
    }
  }

  /// Style for a highlighted row state; `None` for a normal row.
  fn for_state(&self, marked: bool, hovered: bool) -> Option<RowStyle> {
    match (marked, hovered) {
      (true, true) => Some(self.marked_hovered),
      (true, false) => Some(self.marked),
      (false, true) => Some(self.hovered),
      (false, false) => None,
    }
  }
}

fn field_color(field: BookField, theme: &Theme) -> Color {
  let table = &theme.table;
  theme.color(match field {
    BookField::Title => &table.title_field,
    BookField::Authors => &table.authors_field,
    BookField::Series => &table.series_field,
    BookField::Formats => &table.formats_field,
    BookField::Tags => &table.tags_field,
  })
}

/// Split `text` into base and highlighted spans. Ranges count non-whitespace
/// chars (see [`TokenRange`]) and must be sorted and disjoint.
fn highlighted_line(
  text: &str,
  ranges: impl Iterator<Item = TokenRange>,
  style: RowStyle,
) -> Line<'static> {
  let mut ranges = ranges.peekable();
  if ranges.peek().is_none() {
    return Line::from(Span::styled(text.to_string(), style.base));
  }

  let mut current = ranges.next();
  let mut token = 0;
  let mut pending = String::new();
  let mut spans = Vec::new();

  for ch in text.chars() {
    if ch.is_whitespace() {
      pending.push(ch);
      continue;
    }

    if current.is_some_and(|(start, _)| token == start) && !pending.is_empty() {
      spans.push(Span::styled(std::mem::take(&mut pending), style.base));
    }
    pending.push(ch);
    token += 1;
    if current.is_some_and(|(_, end)| token == end) {
      spans.push(Span::styled(std::mem::take(&mut pending), style.highlight));
      current = ranges.next();
    }
  }

  if !pending.is_empty() {
    spans.push(Span::styled(pending, style.base));
  }
  Line::from(spans)
}

#[cfg(test)]
mod tests {
  use super::*;

  fn render(text: &str, ranges: &[TokenRange]) -> Vec<(String, bool)> {
    let style = RowStyle {
      base: Style::default(),
      highlight: Style::default().add_modifier(Modifier::BOLD),
    };
    highlighted_line(text, ranges.iter().copied(), style)
      .spans
      .into_iter()
      .map(|span| (span.content.into_owned(), span.style == style.highlight))
      .collect()
  }

  #[test]
  fn highlights_count_non_whitespace_chars() {
    assert_eq!(
      render(" Harry Potter", &[(3, 7)]),
      [
        (" Har".to_string(), false),
        ("ry Po".to_string(), true),
        ("tter".to_string(), false)
      ]
    );
  }

  #[test]
  fn multibyte_ranges_split_on_char_boundaries() {
    assert_eq!(
      render("張三 & 李四", &[(0, 1), (3, 5)]),
      [
        ("張".to_string(), true),
        ("三 & ".to_string(), false),
        ("李四".to_string(), true)
      ]
    );
  }

  #[test]
  fn out_of_range_highlight_does_not_panic() {
    assert_eq!(
      render("ab", &[(1, 9)]),
      [("a".to_string(), false), ("b".to_string(), false)]
    );
  }

  #[test]
  fn visible_window_follows_focus() {
    // Focus below the window scrolls down just enough.
    assert_eq!(first_visible_row(100, Some(30), 0, 10), 21);
    // Focus above the window scrolls up to it.
    assert_eq!(first_visible_row(100, Some(5), 21, 10), 5);
    // Focus inside the window keeps the scroll position.
    assert_eq!(first_visible_row(100, Some(25), 21, 10), 21);
    // Stale scroll past the end after results shrink.
    assert_eq!(first_visible_row(3, Some(0), 50, 10), 0);
    assert_eq!(first_visible_row(0, None, 7, 10), 0);
    // No room for rows at all.
    assert_eq!(first_visible_row(10, Some(4), 0, 0), 0);
  }
}
