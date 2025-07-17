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
}
