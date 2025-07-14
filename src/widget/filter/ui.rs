use crate::widget::{ControlCode, Filter, Ui};
use anyhow::Result;
use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{Terminal, layout::Rect, prelude::CrosstermBackend};
use std::io::Stdout;

impl Ui for Filter {
    /// main loop
    fn draw_tick(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
        rect: Rect,
    ) -> Result<()> {
        terminal.draw(|f| {
            self.ui_handler.draw(
                f,
                rect,
                &self.input.borrow(),
                &self.filtered_uuids.borrow(),
                &self.books_highlights.borrow(),
                &self.books,
                &mut self.table_state.borrow_mut(),
            )
        })?;
        Ok(())
    }
    fn event_tick(&self, event: &Event) -> Result<()> {
        match event {
            Event::Key(key) => match key.code {
                // // ctrl c to quit the app
                // KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                //     self.send_control_signal(ControlCode::Quit)?;
                // }
                // // esc to quit the app
                // KeyCode::Esc => {
                //     self.send_control_signal(ControlCode::Quit)?;
                // }
                // input
                KeyCode::Char(c) => {
                    self.input.borrow_mut().push(c);
                    self.update()?;
                }
                // delete input
                KeyCode::Backspace => {
                    self.input.borrow_mut().pop();
                    self.update()?;
                }
                // nagivate down
                KeyCode::Down => {
                    self.next_item();
                    self.send_hovered_uuid(
                        self.filtered_uuids.borrow()
                            [self.table_state.borrow().selected().unwrap_or(0)]
                        .clone(),
                    )?;
                }
                // nagivate up
                KeyCode::Up => {
                    self.previous_item();
                    self.send_hovered_uuid(
                        self.filtered_uuids.borrow()
                            [self.table_state.borrow().selected().unwrap_or(0)]
                        .clone(),
                    )?;
                }
                // select item
                KeyCode::Enter => {
                    self.send_selected_uuid(
                        self.filtered_uuids.borrow()
                            [self.table_state.borrow().selected().unwrap_or(0)]
                        .clone(),
                    )?;
                    // exit on open
                    if self.exit_on_open {
                        self.send_control_signal(ControlCode::Quit)?;
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
        Ok(())
    }
}
