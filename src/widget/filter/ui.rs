use crate::widget::{ControlCode, Filter, Ui};
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEventKind};
use ratatui::{Terminal, backend::Backend};
use std::time::Duration;

impl Ui for Filter {
    /// main loop
    fn draw_tick<B: Backend>(&self, terminal: &mut Terminal<B>) -> Result<()> {
        terminal.draw(|f| {
            self.ui_handler.draw(
                f,
                &self.input.borrow(),
                &self.filtered_uuids.borrow(),
                &self.books_highlights.borrow(),
                &self.books,
                &mut self.table_state.borrow_mut(),
            )
        })?;

        Ok(())
    }
    fn event_tick(&self) -> Result<()> {
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    // ctrl c to quit the app
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        self.send_control_signal(ControlCode::Quit)?;
                    }
                    // esc to quit the app
                    KeyCode::Esc => {
                        self.send_control_signal(ControlCode::Quit)?;
                    }
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
        }
        Ok(())
    }
}
