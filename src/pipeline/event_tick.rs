use std::time::Duration;

use crate::pipeline::Pipeline;
use anyhow::{Result, Context};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::layout::Rect;

impl Pipeline {
    pub fn event_tick(&self) -> Result<Option<Event>> {
        if event::poll(Duration::from_millis(250))? {
            let event = event::read()?;
            match event {
                Event::Resize(width, height) => {
                    self.update_ui_rects(Rect::new(0, 0, width, height)).context("failed to update ui rects")?;
                }

                Event::Key(key) => match key.code {
                    // ctrl c to quit the app
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        *self.should_exit.lock().unwrap() = true;
                    }
                    // esc to quit the app
                    KeyCode::Esc => {
                        *self.should_exit.lock().unwrap() = true;
                    }
                    _ => {
                        return Ok(Some(event));
                    }
                },
                _ => {
                    return Ok(Some(event));
                }
            }
        }
        Ok(None)
    }

    /// Collect all available events in a batch (blocking for the first, then draining the rest)
    pub fn event_tick_batch(&self) -> Result<Vec<Event>> {
        let mut events = Vec::new();

        // Block for the first event
        if event::poll(Duration::from_millis(250))? {
            let event = event::read()?;
            match &event {
                Event::Resize(width, height) => {
                    self.update_ui_rects(Rect::new(0, 0, *width, *height)).context("failed to update ui rects")?;
                }
                Event::Key(key) => match key.code {
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        *self.should_exit.lock().unwrap() = true;
                    }
                    KeyCode::Esc => {
                        *self.should_exit.lock().unwrap() = true;
                    }
                    _ => {
                        events.push(event);
                    }
                },
                _ => {
                    events.push(event);
                }
            }

            // Drain all other available events (non-blocking)
            while event::poll(Duration::from_millis(0))? {
                let event = event::read()?;
                match &event {
                    Event::Resize(width, height) => {
                        self.update_ui_rects(Rect::new(0, 0, *width, *height)).context("failed to update ui rects")?;
                    }
                    Event::Key(key) => match key.code {
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            *self.should_exit.lock().unwrap() = true;
                        }
                        KeyCode::Esc => {
                            *self.should_exit.lock().unwrap() = true;
                        }
                        _ => {
                            events.push(event);
                        }
                    },
                    _ => {
                        events.push(event);
                    }
                }
            }
        }

        Ok(events)
    }
}
