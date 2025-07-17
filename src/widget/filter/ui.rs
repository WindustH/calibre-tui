use crate::widget::{Filter, Ui};
use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, MouseEventKind};
use ratatui::{Terminal, layout::Rect, prelude::CrosstermBackend};
use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};

impl Ui for Filter {
    /// main loop
    fn draw_tick(
        &self,
        terminal: Arc<Mutex<Terminal<CrosstermBackend<Stdout>>>>,
        rect: Rect,
    ) -> Result<()> {
        terminal.lock().unwrap().draw(|f| {
            self.ui_handler.draw(
                f,
                rect,
                &self.input.lock().unwrap(),
                &self.filtered_uuids.lock().unwrap(),
                &self.books_highlights.lock().unwrap(),
                &self.books,
                &mut self.table_state.lock().unwrap(),
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
                    self.input.lock().unwrap().push(c);
                    self.update()
                        .context("failed to update results after pushing a char to input")?;
                }
                // delete input
                KeyCode::Backspace => {
                    self.input.lock().unwrap().pop();
                    self.update()
                        .context("failed to update results after popping a char from input")?;
                }
                // nagivate down
                KeyCode::Down => {
                    self.next_item();
                    self.send_hovered_uuid(
                        self.filtered_uuids.lock().unwrap()
                            [self.table_state.lock().unwrap().selected().unwrap_or(0)]
                        .clone(),
                    )
                    .context("failed to send hovered uuid when navigating down")?;
                }
                // nagivate up
                KeyCode::Up => {
                    self.previous_item();
                    self.send_hovered_uuid(
                        self.filtered_uuids.lock().unwrap()
                            [self.table_state.lock().unwrap().selected().unwrap_or(0)]
                        .clone(),
                    )
                    .context("failed to send hovered uuid when navigating up")?;
                }
                // select item
                KeyCode::Enter => {
                    self.send_selected_uuid(
                        self.filtered_uuids.lock().unwrap()
                            [self.table_state.lock().unwrap().selected().unwrap_or(0)]
                        .clone(),
                    )
                    .context("failed to send selected uuid when pressing enter")?;
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
