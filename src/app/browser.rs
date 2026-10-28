//! Browser state: the search input, its sorted results, the focused row, and
//! the marked (multi-selected) books.

use crate::search::SearchResult;
use crate::ui::TableViewport;
use std::collections::BTreeSet;

pub struct Browser {
  input: String,
  results: Vec<SearchResult>,
  /// Focused row in `results`; `None` only when there are no results.
  focus: Option<usize>,
  /// First visible row, as last laid out by the UI.
  scroll: usize,
  page_size: usize,
  /// Marked book indices; kept while the query changes.
  marked: BTreeSet<usize>,
}

impl Browser {
  pub fn new() -> Self {
    Self {
      input: String::new(),
      results: Vec::new(),
      focus: None,
      scroll: 0,
      page_size: 20,
      marked: BTreeSet::new(),
    }
  }

  pub fn input(&self) -> &str {
    &self.input
  }

  pub fn push_input(&mut self, ch: char) {
    self.input.push(ch);
  }

  /// Delete the last input char; returns whether the input changed.
  pub fn pop_input(&mut self) -> bool {
    self.input.pop().is_some()
  }

  pub fn results(&self) -> &[SearchResult] {
    &self.results
  }

  pub fn focus(&self) -> Option<usize> {
    self.focus
  }

  pub fn scroll(&self) -> usize {
    self.scroll
  }

  pub fn marked(&self) -> &BTreeSet<usize> {
    &self.marked
  }

  /// Replace the results after a new search and focus the first row.
  pub fn set_results(&mut self, results: Vec<SearchResult>) {
    self.results = results;
    self.focus = (!self.results.is_empty()).then_some(0);
  }

  /// Re-sort the results in place, keeping focus on the same book.
  pub fn reorder(&mut self, sort: impl FnOnce(&mut [SearchResult])) {
    let focused = self.focused_book();
    sort(&mut self.results);
    if let Some(row) = focused.and_then(|book| self.row_of(book)) {
      self.focus = Some(row);
    }
  }

  pub fn set_viewport(&mut self, viewport: TableViewport) {
    self.scroll = viewport.first_row;
    self.page_size = viewport.rows.max(1);
  }

  pub fn move_up(&mut self) {
    self.move_focus(|focus, last| match focus {
      Some(0) | None => last,
      Some(row) => row - 1,
    });
  }

  pub fn move_down(&mut self) {
    self.move_focus(|focus, last| match focus {
      Some(row) if row < last => row + 1,
      _ => 0,
    });
  }

  pub fn page_up(&mut self) {
    let page = self.page_size;
    self.move_focus(|focus, _| focus.unwrap_or(0).saturating_sub(page));
  }

  pub fn page_down(&mut self) {
    let page = self.page_size;
    self.move_focus(|focus, _| focus.unwrap_or(0).saturating_add(page));
  }

  pub fn jump_start(&mut self) {
    self.move_focus(|_, _| 0);
  }

  pub fn jump_end(&mut self) {
    self.move_focus(|_, last| last);
  }

  /// Toggle the mark on the focused book and move to the next row.
  pub fn toggle_mark(&mut self) {
    let Some(book) = self.focused_book() else {
      return;
    };
    if !self.marked.insert(book) {
      self.marked.remove(&book);
    }
    self.move_down();
  }

  pub fn mark_all(&mut self) {
    self
      .marked
      .extend(self.results.iter().map(|result| result.book_index));
  }

  pub fn clear_marks(&mut self) {
    self.marked.clear();
  }

  pub fn focused_book(&self) -> Option<usize> {
    self
      .focus
      .and_then(|row| self.results.get(row))
      .map(|result| result.book_index)
  }

  /// Books an action applies to: the marked books, or the focused one.
  pub fn target_books(&self) -> Vec<usize> {
    if self.marked.is_empty() {
      self.focused_book().into_iter().collect()
    } else {
      self.marked.iter().copied().collect()
    }
  }

  fn row_of(&self, book: usize) -> Option<usize> {
    self
      .results
      .iter()
      .position(|result| result.book_index == book)
  }

  /// Move focus to `target(focus, last_row)`, clamped to the results.
  fn move_focus(&mut self, target: impl FnOnce(Option<usize>, usize) -> usize) {
    self.focus = self
      .results
      .len()
      .checked_sub(1)
      .map(|last| target(self.focus, last).min(last));
  }
}
