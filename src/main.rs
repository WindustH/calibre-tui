mod app;
mod config;
mod db;
mod pinyin;
mod ui;

use anyhow::{Context, Result};
use app::App;
// Add clap for command-line argument parsing.
// Please add `clap = { version = "4.5.11", features = ["derive"] }` to your Cargo.toml
use clap::Parser;
use config::{find_calibre_library_path, load_config};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{io, time::Duration};

/// A fast and customizable TUI for your Calibre library.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Exit the application immediately after opening a book
    // RENAMED: from --exit-after-open to --exit-on-open
    #[arg(long)]
    exit_on_open: bool,
}

fn main() -> Result<()> {
    // --- Argument Parsing ---
    let args = Args::parse();

    // --- Setup ---
    let config = load_config()?;
    let library_path = find_calibre_library_path(&config)
        .context("Could not find Calibre library. Please specify `library_path` in config.toml or ensure it's in a standard location.")?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- App ---
    // Use the renamed argument.
    let mut app = App::new(library_path, config, args.exit_on_open)?;
    let res = run_app(&mut terminal, &mut app);

    // --- Cleanup ---
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Application error: {:?}", err)
    }

    Ok(())
}

/// 主应用循环
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        if app.should_quit {
            return Ok(());
        }

        terminal.draw(|f| ui::draw(f, app))?;

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
                        KeyCode::Enter => app.open_selected_book(),
                        _ => {}
                    }
                }
                Event::Mouse(mouse_event) => match mouse_event.kind {
                    MouseEventKind::ScrollDown => app.next_item(),
                    MouseEventKind::ScrollUp => app.previous_item(),
                    _ => {}
                },
                _ => {} // Ignore other events
            }
        }
    }
}
