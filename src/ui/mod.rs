//! Rendering: search/command box, completion list, book table, footer, and
//! the key help popup. Drawing reads a [`View`] and never mutates app state.

mod table;

pub use table::{BookTable, TableViewport};

use crate::layout::Layout;
use crate::theme::Theme;
use framework_tui::{
  CommandCompletion, CompletionListStyle, KeyHelpDialogStyle, KeyHelpEntry, KeyHint, KeyHintsStyle,
  PopupDialogStyle, Prompt, PromptLineStyle, completion_rows, default_completion_selected_style,
  draw_completion_list, draw_key_help_dialog, draw_key_hints, draw_prompt_line, key_hint_columns,
  key_hint_rows,
};
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout as TuiLayout, Rect},
  style::{Modifier, Style},
  text::Span,
  widgets::{Block, BorderType, Borders, Paragraph},
};
use unicode_width::UnicodeWidthStr;

pub struct View<'a> {
  pub theme: &'a Theme,
  pub layout: &'a Layout,
  pub input: &'a str,
  pub table: BookTable<'a>,
  pub sort_label: &'a str,
  pub prompt: Option<&'a Prompt>,
  pub completion: Option<&'a CommandCompletion>,
  pub key_hints: &'a [KeyHint],
  pub key_help: Option<&'a [KeyHelpEntry]>,
  pub message: Option<&'a str>,
}

/// Draw the whole screen; returns the table rows that were shown.
pub fn draw(frame: &mut Frame, view: &View<'_>) -> TableViewport {
  let area = frame.area();
  let theme = view.theme;
  frame.render_widget(
    Block::default().style(
      Style::default()
        .fg(theme.color(&theme.foreground))
        .bg(theme.color(&theme.background)),
    ),
    area,
  );

  let hint_rows = if view.key_hints.is_empty() {
    0
  } else {
    let columns = key_hint_columns(theme.footer.which_key_columns, area.width);
    key_hint_rows(view.key_hints.len(), columns)
  };
  let footer_height = hint_rows.max(u16::from(view.message.is_some()));
  let completion_height = if view.prompt.is_some() {
    completion_rows(view.completion, 5)
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

  match view.prompt {
    Some(prompt) => draw_command_input(frame, chunks[0], prompt, view.completion, theme),
    None => draw_search_input(frame, chunks[0], view),
  }
  let viewport = table::draw_table(frame, chunks[2], &view.table, view.layout, theme);
  if view.prompt.is_some()
    && let Some(completion) = view.completion
  {
    draw_command_completion(frame, chunks[1], completion, theme);
  }
  draw_footer(frame, chunks[3], view.key_hints, view.message, theme);

  if let Some(entries) = view.key_help {
    draw_key_help(frame, area, entries, theme);
  }
  viewport
}

fn draw_search_input(frame: &mut Frame, area: Rect, view: &View<'_>) {
  let theme = view.theme;
  let title = format!(
    " Search [{} selected] [sort: {}] ",
    view.table.marked.len(),
    view.sort_label
  );
  let block = Block::default()
    .borders(Borders::ALL)
    .border_style(Style::default().fg(theme.color(&theme.search.border)))
    .border_type(BorderType::Rounded)
    .title(Span::styled(
      title,
      Style::default().fg(theme.color(&theme.search.title)),
    ));
  let input_box = Paragraph::new(view.input)
    .style(
      Style::default()
        .fg(theme.color(&theme.search.text))
        .bg(theme.color(&theme.background)),
    )
    .block(block);
  frame.render_widget(input_box, area);

  let max_cursor_width = area.width.saturating_sub(2);
  let cursor_width = u16::try_from(view.input.width())
    .unwrap_or(u16::MAX)
    .min(max_cursor_width);
  frame.set_cursor_position((area.x + cursor_width + 1, area.y + 1));
}

fn draw_command_input(
  frame: &mut Frame,
  area: Rect,
  prompt: &Prompt,
  completion: Option<&CommandCompletion>,
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
  let background = theme.color(&theme.background);
  let style = PromptLineStyle {
    base: Style::default()
      .fg(theme.color(&theme.command.text))
      .bg(background),
    prefix: Style::default()
      .fg(theme.color(&theme.command.prefix))
      .bg(background)
      .add_modifier(Modifier::BOLD),
    suggestion: Style::default()
      .fg(theme.color(&theme.command.suggestion))
      .bg(background),
  };
  draw_prompt_line(frame, prompt, completion, inner, &style);
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

  let footer = &theme.footer;
  if !key_hints.is_empty() {
    let base = Style::default()
      .fg(theme.color(&footer.which_key_foreground))
      .bg(theme.color(&footer.which_key_background));
    let style = KeyHintsStyle {
      base,
      key: base
        .fg(theme.color(&footer.which_key_key))
        .add_modifier(Modifier::BOLD),
      separator: base.fg(theme.color(&footer.which_key_separator)),
      description: base.fg(theme.color(&footer.which_key_description)),
      separator_text: footer.which_key_separator_text.clone(),
      columns: footer.which_key_columns,
    };
    draw_key_hints(frame, key_hints, area, &style);
  } else if let Some(message) = message {
    frame.render_widget(
      Paragraph::new(message).style(
        Style::default()
          .fg(theme.color(&footer.message))
          .bg(theme.color(&theme.background)),
      ),
      area,
    );
  }
}

fn draw_command_completion(
  frame: &mut Frame,
  area: Rect,
  completion: &CommandCompletion,
  theme: &Theme,
) {
  let colors = &theme.completion;
  let style = CompletionListStyle {
    base: Style::default()
      .fg(theme.color(&colors.foreground))
      .bg(theme.color(&colors.background)),
    selected: default_completion_selected_style()
      .fg(theme.color(&colors.selected_foreground))
      .bg(theme.color(&colors.selected_background)),
  };
  draw_completion_list(frame, completion, area, &style);
}

fn draw_key_help(frame: &mut Frame, area: Rect, entries: &[KeyHelpEntry], theme: &Theme) {
  let help = &theme.help;
  let background = theme.color(&help.background);
  let defaults = KeyHelpDialogStyle::default();
  let style = KeyHelpDialogStyle {
    key: Style::default()
      .fg(theme.color(&help.key))
      .bg(background)
      .add_modifier(Modifier::BOLD),
    description: Style::default()
      .fg(theme.color(&help.description))
      .bg(background),
    muted: Style::default().fg(theme.color(&help.muted)).bg(background),
    popup: PopupDialogStyle {
      base: Style::default()
        .fg(theme.color(&theme.foreground))
        .bg(background),
      border: Style::default().fg(theme.color(&help.border)),
      ..defaults.popup
    },
    ..defaults
  };
  draw_key_help_dialog(frame, area, "Key Bindings", entries, &style);
}
