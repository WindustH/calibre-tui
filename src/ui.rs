use crate::filter::{BookHighlights, HighlightRanges, SearchResult};
use crate::layout::{BookField, Layout};
use crate::theme::Theme;
use crate::utils::book::Book;
use framework_tui::{
  CommandCompletion, CompletionListStyle, KeyHelpDialogStyle, KeyHint, KeyHintsStyle, Prompt,
  PromptLineStyle, completion_rows, default_completion_selected_style, draw_completion_list,
  draw_key_help_dialog, draw_key_hints, draw_prompt_line, key_hint_columns, key_hint_rows,
};
use ratatui::layout::Rect;
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout as TuiLayout},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table, TableState},
};
use std::collections::BTreeSet;
use unicode_width::UnicodeWidthStr;

pub struct DrawState<'a> {
  pub input: &'a str,
  pub books: &'a [Book],
  pub results: &'a [SearchResult],
  pub table_state: &'a mut TableState,
  pub selected_book_indices: &'a BTreeSet<usize>,
  pub layout: &'a Layout,
  pub theme: &'a Theme,
  pub prompt: Option<&'a Prompt>,
  pub command_completion: Option<&'a CommandCompletion>,
  pub key_hints: &'a [KeyHint],
  pub key_help_entries: Option<&'a [framework_tui::KeyHelpEntry]>,
  pub message: Option<&'a str>,
  pub sort_label: &'a str,
}

pub fn draw(frame: &mut Frame, area: Rect, state: DrawState<'_>) {
  let DrawState {
    input,
    books,
    results,
    table_state,
    selected_book_indices,
    layout,
    theme,
    prompt,
    command_completion,
    key_hints,
    key_help_entries,
    message,
    sort_label,
  } = state;

  frame.render_widget(
    Block::default().style(
      Style::default()
        .fg(theme.color(&theme.foreground))
        .bg(theme.color(&theme.background)),
    ),
    area,
  );

  let hint_rows = if key_hints.is_empty() {
    0
  } else {
    let columns = key_hint_columns(theme.footer.which_key_columns, area.width);
    key_hint_rows(key_hints.len(), columns)
  };
  let footer_height = hint_rows.max(u16::from(message.is_some()));
  let completion_height = if prompt.is_some() {
    completion_rows(command_completion, 5)
  } else {
    0
  };

  let chunks = TuiLayout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints([
      Constraint::Length(3),
      Constraint::Length(completion_height),
      Constraint::Min(0),
      Constraint::Length(footer_height),
    ])
    .split(area);

  draw_input_box(
    frame,
    chunks[0],
    input,
    selected_book_indices.len(),
    theme,
    prompt,
    command_completion,
    sort_label,
  );
  draw_table(
    frame,
    chunks[2],
    books,
    results,
    table_state,
    selected_book_indices,
    layout,
    theme,
  );
  draw_command_completion(frame, chunks[1], command_completion, theme);
  draw_footer(frame, chunks[3], key_hints, message, theme);

  if let Some(entries) = key_help_entries {
    draw_key_help(frame, area, entries, theme);
  }
}

fn draw_input_box(
  frame: &mut Frame,
  area: Rect,
  input: &str,
  selected_count: usize,
  theme: &Theme,
  prompt: Option<&Prompt>,
  command_completion: Option<&CommandCompletion>,
  sort_label: &str,
) {
  if let Some(prompt) = prompt {
    draw_command_input(frame, area, prompt, command_completion, theme);
    return;
  }

  let title = format!(" Search [{selected_count} selected] [sort: {sort_label}] ");
  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme.color(&theme.search.border)))
    .border_type(BorderType::Rounded)
    .title(Span::styled(
      title,
      Style::default().fg(theme.color(&theme.search.title)),
    ));
  let input_box = Paragraph::new(input)
    .style(
      Style::default()
        .fg(theme.color(&theme.search.text))
        .bg(theme.color(&theme.background)),
    )
    .block(block);
  frame.render_widget(input_box, area);

  let max_cursor_width = area.width.saturating_sub(2) as usize;
  let cursor_width = input.width().min(max_cursor_width);
  frame.set_cursor_position((area.x + cursor_width as u16 + 1, area.y + 1));
}

fn draw_command_input(
  frame: &mut Frame,
  area: Rect,
  prompt: &Prompt,
  command_completion: Option<&CommandCompletion>,
  theme: &Theme,
) {
  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme.color(&theme.command.border)))
    .border_type(BorderType::Rounded)
    .title(Span::styled(
      " Command ",
      Style::default().fg(theme.color(&theme.command.title)),
    ));
  let inner = block.inner(area);
  frame.render_widget(block, area);
  let style = PromptLineStyle {
    base: Style::default()
      .fg(theme.color(&theme.command.text))
      .bg(theme.color(&theme.background)),
    prefix: Style::default()
      .fg(theme.color(&theme.command.prefix))
      .bg(theme.color(&theme.background))
      .add_modifier(Modifier::BOLD),
    suggestion: Style::default()
      .fg(theme.color(&theme.command.suggestion))
      .bg(theme.color(&theme.background)),
  };
  let _ = draw_prompt_line(frame, prompt, command_completion, inner, &style);
}

fn draw_footer(
  frame: &mut Frame,
  area: Rect,
  key_hints: &[KeyHint],
  message: Option<&str>,
  theme: &Theme,
) {
  if area.height == 0 {
    return;
  }

  if !key_hints.is_empty() {
    let base = Style::default()
      .fg(theme.color(&theme.footer.which_key_foreground))
      .bg(theme.color(&theme.footer.which_key_background));
    let style = KeyHintsStyle {
      base,
      key: base
        .fg(theme.color(&theme.footer.which_key_key))
        .add_modifier(Modifier::BOLD),
      separator: base.fg(theme.color(&theme.footer.which_key_separator)),
      description: base.fg(theme.color(&theme.footer.which_key_description)),
      separator_text: theme.footer.which_key_separator_text.clone(),
      columns: theme.footer.which_key_columns,
    };
    draw_key_hints(frame, key_hints, area, &style);
    return;
  }

  if let Some(message) = message {
    frame.render_widget(
      Paragraph::new(message.to_string()).style(
        Style::default()
          .fg(theme.color(&theme.footer.message))
          .bg(theme.color(&theme.background)),
      ),
      area,
    );
  }
}

fn draw_command_completion(
  frame: &mut Frame,
  area: Rect,
  command_completion: Option<&CommandCompletion>,
  theme: &Theme,
) {
  let Some(completion) = command_completion else {
    return;
  };
  let style = CompletionListStyle {
    base: Style::default()
      .fg(theme.color(&theme.completion.foreground))
      .bg(theme.color(&theme.completion.background)),
    selected: default_completion_selected_style()
      .fg(theme.color(&theme.completion.selected_foreground))
      .bg(theme.color(&theme.completion.selected_background)),
  };
  draw_completion_list(frame, completion, area, &style);
}

fn draw_key_help(
  frame: &mut Frame,
  area: Rect,
  entries: &[framework_tui::KeyHelpEntry],
  theme: &Theme,
) {
  let style = KeyHelpDialogStyle {
    key: Style::default()
      .fg(theme.color(&theme.help.key))
      .bg(theme.color(&theme.help.background))
      .add_modifier(Modifier::BOLD),
    description: Style::default()
      .fg(theme.color(&theme.help.description))
      .bg(theme.color(&theme.help.background)),
    muted: Style::default()
      .fg(theme.color(&theme.help.muted))
      .bg(theme.color(&theme.help.background)),
    popup: framework_tui::PopupDialogStyle {
      base: Style::default()
        .fg(theme.color(&theme.foreground))
        .bg(theme.color(&theme.help.background)),
      border: Style::default().fg(theme.color(&theme.help.border)),
      ..KeyHelpDialogStyle::default().popup
    },
    ..KeyHelpDialogStyle::default()
  };
  let _ = draw_key_help_dialog(frame, area, "Key Bindings", entries, &style);
}

fn draw_table(
  frame: &mut Frame,
  area: Rect,
  books: &[Book],
  results: &[SearchResult],
  table_state: &mut TableState,
  selected_book_indices: &BTreeSet<usize>,
  layout: &Layout,
  theme: &Theme,
) {
  let columns = layout.visible_columns().collect::<Vec<_>>();
  let header = Row::new(columns.iter().map(|column| {
    Cell::from(column.label.clone()).style(
      Style::default()
        .fg(theme.color(&theme.table.header))
        .bg(theme.color(&theme.background))
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

    Row::new(columns.iter().map(|column| {
      let (text, ranges) = field_text_and_highlights(book, &result.highlights, column.field);
      let base_style = if is_marked && is_hovered {
        Style::default()
          .fg(theme.color(&theme.row.selected_hover_foreground))
          .bg(theme.color(&theme.row.selected_hover_background))
      } else if is_marked {
        Style::default()
          .fg(theme.color(&theme.row.selected_foreground))
          .bg(theme.color(&theme.row.selected_background))
      } else if is_hovered {
        Style::default()
          .fg(theme.color(&theme.row.hover_foreground))
          .bg(theme.color(&theme.row.hover_background))
      } else {
        Style::default()
          .fg(field_color(column.field, theme))
          .bg(theme.color(&theme.background))
      };
      let highlight_style = if is_marked && is_hovered {
        Style::default()
          .fg(theme.color(&theme.highlight.selected_hover))
          .bg(theme.color(&theme.row.selected_hover_background))
          .add_modifier(Modifier::BOLD)
      } else if is_marked {
        Style::default()
          .fg(theme.color(&theme.highlight.selected))
          .bg(theme.color(&theme.row.selected_background))
          .add_modifier(Modifier::BOLD)
      } else if is_hovered {
        Style::default()
          .fg(theme.color(&theme.highlight.hover))
          .bg(theme.color(&theme.row.hover_background))
          .add_modifier(Modifier::BOLD)
      } else {
        Style::default()
          .fg(theme.color(&theme.highlight.normal))
          .bg(theme.color(&theme.background))
          .add_modifier(Modifier::BOLD)
      };

      Cell::from(highlighted_line(&text, ranges, base_style, highlight_style)).style(base_style)
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

  let table = Table::new(rows, widths)
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

  frame.render_stateful_widget(table, area, table_state);
}

fn field_text_and_highlights<'a>(
  book: &'a Book,
  highlights: &'a BookHighlights,
  field: BookField,
) -> (String, &'a HighlightRanges) {
  (field_text(book, field), highlights.ranges(field))
}

fn field_text(book: &Book, field: BookField) -> String {
  match field {
    BookField::Title => book.title.clone(),
    BookField::Authors => book.authors.join(" & "),
    BookField::Series => book.series.clone(),
    BookField::Formats => book.formats.join(", "),
    BookField::Tags => book.tags.join(", "),
  }
}

fn field_color(field: BookField, theme: &Theme) -> Color {
  match field {
    BookField::Title => theme.color(&theme.table.title_field),
    BookField::Authors => theme.color(&theme.table.authors_field),
    BookField::Series => theme.color(&theme.table.series_field),
    BookField::Formats => theme.color(&theme.table.formats_field),
    BookField::Tags => theme.color(&theme.table.tags_field),
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
