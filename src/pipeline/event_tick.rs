use std::time::Duration;

use crate::pipeline::Pipeline;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

impl Pipeline {
    pub fn event_tick(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    // ctrl c to quit the app
                    KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                        self.should_exit = true;
                    }
                    // esc to quit the app
                    KeyCode::Esc => {
                        self.should_exit = true;
                    }
                    _ => {}
                },
                _ => {} // ignore
            }
        }
        Ok(())
    }
}
