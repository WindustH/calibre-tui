mod widget;
mod config;
mod i18n;
mod ui;
mod utils;
mod pipeline;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

use crate::widget::Ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    exit_on_open: bool,
}

fn main() -> Result<()> {
    // parse arguements
    let args = Args::parse();

    // setup
    let config = config::load_config()?;
    let database = utils::db::load_books_from_db(&config.app.library_path)?;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // app

    let app = widget::Filter::new(
        &config,
        args.exit_on_open
    )?;
    let res: Result<()> = loop {
        app.draw_tick(&mut terminal)?;

    };

    // cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // if let Err(err) = res {
    //     println!("application error: {:?}", err)
    // }

    Ok(())
}
