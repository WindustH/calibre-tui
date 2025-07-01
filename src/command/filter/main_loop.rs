use crossterm::{
    event::{self,Event, KeyCode, KeyModifiers, MouseEventKind},
};
use ratatui::{
    backend::Backend,
    Terminal,
};

use std::{io, time::Duration};

use crate::command;

/// main loop
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut command::Filter) -> io::Result<()> {
    loop {
        if app.should_quit {
            return Ok(());
        }

        terminal.draw(|f| ui::filter::draw(f, app))?;

        if event::poll(Duration::from_millis(250))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                            app.should_quit = true;
                        }
                        KeyCode::Esc => {
                            app.should_quit = true;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                            app.filter_books();
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                            app.filter_books();
                        }
                        KeyCode::Down => app.next_item(),
                        KeyCode::Up => app.previous_item(),
                        KeyCode::Enter => {
                            let hovered_book=vec![app.get_hovered_book_relative_path()];
                            command::open::open_books();
                        }
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::ScrollDown => app.next_item(),
                    MouseEventKind::ScrollUp => app.previous_item(),
                    _ => {}
                },
                _ => {} // ignore
            }
        }
    }
}
