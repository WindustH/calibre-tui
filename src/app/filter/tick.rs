use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::{Terminal, backend::Backend};
use std::time::Duration;

impl<'a> super::super::Ui for super::super::Filter<'a> {
    /// main loop
    fn tick<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            if self.should_quit {
                return Ok(());
            }
            terminal.draw(|f| {
                self.ui_handler.draw(
                    f,
                    &self.input,
                    &self.filtered_uuids,
                    &self.books_highlights,
                    self.database,
                    &mut self.table_state,
                )
            })?;

            if event::poll(Duration::from_millis(250))? {
                match event::read()? {
                    Event::Key(key) => match key.code {
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            self.should_quit = true;
                        }
                        KeyCode::Esc => {
                            self.should_quit = true;
                        }
                        KeyCode::Char(c) => {
                            self.input.push(c);
                            self.update_filtered_books_and_create_highlights()?;
                        }
                        KeyCode::Backspace => {
                            self.input.pop();
                            self.update_filtered_books_and_create_highlights()?;
                        }
                        KeyCode::Down => self.next_item(),
                        KeyCode::Up => self.previous_item(),
                        KeyCode::Enter => {
                            let uuids = match self.get_hovered() {
                                Some(uuid) => vec![uuid],
                                None => vec![],
                            };
                            crate::app::open::open_books(self.database, &uuids);
                            // exit on open
                            if self.exit_on_open {
                                self.should_quit = true;
                            }
                        }
                        _ => {}
                    },
                    Event::Mouse(mouse_event) => match mouse_event.kind {
                        MouseEventKind::ScrollDown => self.next_item(),
                        MouseEventKind::ScrollUp => self.previous_item(),
                        _ => {}
                    },
                    _ => {} // ignore
                }
            }
        }
    }
}
