mod config;
mod i18n;
mod pipeline;
mod ui;
mod utils;
mod widget;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    collections::HashMap,
    panic,
    sync::{
        Arc, Mutex,
        mpsc::{self, TryRecvError},
    },
    time::{Duration, Instant},
};
use std::{io, thread};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    exit_on_open: bool,
}

enum ControlCode {
    Quit,
    Tick,
}
fn main() -> Result<()> {
    // parse arguements
    let args = Args::parse();

    // setup
    let config = match config::load_config() {
        Ok(config) => config,
        Err(err) => {
            eprintln!("failed to load configuration: {}", err);
            return Err(err.into());
        }
    };
    // try to enable raw mode
    match enable_raw_mode() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("failed to enable raw mode: {}", err);
            return Err(err.into());
        }
    }

    let mut stdout = io::stdout();
    match execute!(stdout, EnterAlternateScreen, EnableMouseCapture) {
        Ok(_) => {}
        Err(err) => {
            eprintln!("failed to enter alternate screen: {}", err);
            return Err(err.into());
        }
    }

    let backend = CrosstermBackend::new(stdout);

    let terminal = Arc::new(Mutex::new(match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(err) => {
            eprintln!("failed to create terminal: {}", err);
            return Err(err.into());
        }
    }));

    // pipeline

    let pipeline = Arc::new(pipeline::Pipeline::new(&config, "filter-and-open"));
    pipeline
        .update_ui_rects(terminal.lock().unwrap().size()?)
        .context("failed to update ui rects")?;

    let mut event_channels = HashMap::new();
    let mut widget_tick_loop_control_code_channels = HashMap::new();
    let mut draw_loop_control_code_channels = HashMap::new();
    let mut event_loop_control_code_channels = HashMap::new();
    // initialize event channels for ui widgets
    for (widget_id, _) in pipeline.ui_rects.lock().unwrap().iter() {
        if let Some(widget) = pipeline.widgets.get(widget_id) {
            if let Some(ui) = widget.as_ui() {
                event_channels.insert(widget_id.clone(), mpsc::channel::<Event>());
                draw_loop_control_code_channels
                    .insert(widget_id.clone(), mpsc::channel::<ControlCode>());
                event_loop_control_code_channels
                    .insert(widget_id.clone(), mpsc::channel::<ControlCode>());
            } else {
                panic!("widget with id {} is not a ui widget", widget_id);
            }
        } else {
            panic!("widget with id {} not found", widget_id);
        }
    }
    // initialize control code channels for widgets
    for (widget_id, widget) in pipeline.widgets.iter() {
        widget_tick_loop_control_code_channels
            .insert(widget_id.clone(), mpsc::channel::<ControlCode>());
    }
    // pipeline event loop
    let pipeline_clone = pipeline.clone();
    let event_senders = event_channels
        .iter()
        .map(|(widget_id, (sender, _))| (widget_id.clone(), sender.clone()))
        .collect::<HashMap<String, mpsc::Sender<Event>>>();
    let control_code_senders_for_widget_tick_loop = widget_tick_loop_control_code_channels
        .iter()
        .map(|(widget_id, (sender, _))| (widget_id.clone(), sender.clone()))
        .collect::<HashMap<String, mpsc::Sender<ControlCode>>>();
    let control_code_senders_for_draw_loop = draw_loop_control_code_channels
        .iter()
        .map(|(widget_id, (sender, _))| (widget_id.clone(), sender.clone()))
        .collect::<HashMap<String, mpsc::Sender<ControlCode>>>();
    let control_code_senders_for_event_loop = event_loop_control_code_channels
        .iter()
        .map(|(widget_id, (sender, _))| (widget_id.clone(), sender.clone()))
        .collect::<HashMap<String, mpsc::Sender<ControlCode>>>();
    let pipeline_event_loop_thread_handle = thread::spawn(move || {
        loop {
            let event = pipeline_clone.event_tick().unwrap_or_else(|err| {
                panic!("failed to handle event: {}", err);
            });
            // try to send event to widgets
            if let Some(e) = event {
                for (widget_id, sender) in &event_senders {
                    if let Err(err) = sender.send(e.clone()) {
                        panic!("failed to send event to widget {}: {}", widget_id, err);
                    }
                }
            }
            if pipeline_clone.should_exit.lock().unwrap().clone() {
                // try to send quit control code to all widgets
                for (_, sender) in &control_code_senders_for_widget_tick_loop {
                    if let Err(err) = sender.send(ControlCode::Quit) {
                        panic!("failed to send quit control code to widget: {}", err);
                    }
                }
                for (_, sender) in &control_code_senders_for_draw_loop {
                    if let Err(err) = sender.send(ControlCode::Quit) {
                        panic!("failed to send quit control code to widget: {}", err);
                    }
                }
                for (_, sender) in &control_code_senders_for_event_loop {
                    if let Err(err) = sender.send(ControlCode::Quit) {
                        panic!("failed to send quit control code to widget: {}", err);
                    }
                }
                break;
            }
        }
    });
    let mut widget_tick_loop_thread_handles = HashMap::new();
    let mut draw_loop_thread_handles = HashMap::new();
    let mut event_loop_thread_handles = HashMap::new();

    // widget tick loop for all widgets
    for (widget_id, widget) in pipeline.widgets.iter() {
        if let Some((_, receiver)) = widget_tick_loop_control_code_channels.remove(widget_id) {
            let widget_clone = widget.clone();
            let thread_handle = thread::spawn(move || {
                loop {
                    widget_clone.tick();
                    match receiver.try_recv() {
                        Ok(ControlCode::Quit) => break,
                        _ => {}
                    }
                }
            });
            widget_tick_loop_thread_handles.insert(widget_id.clone(), thread_handle);
        } else {
            panic!("widget {} not found in control code channels", widget_id);
        }
    }
    let max_refresh_rate: u16 = 60;
    // draw loop and event loop for ui widgets
    let ui_rects = pipeline.ui_rects.lock().unwrap().clone();
    for (widget_id, rect) in ui_rects.iter() {
        // draw loop
        if let Some((_, receiver)) = draw_loop_control_code_channels.remove(widget_id) {
            let widget_clone = pipeline.widgets.get(widget_id).unwrap().clone();
            let rect_clone = (*rect).clone();
            let terminal_clone = terminal.clone();
            let widget_id_clone = widget_id.clone();
            let thread_handle = thread::spawn(move || {
                loop {
                    let start = Instant::now();
                    if let Some(ui) = widget_clone.as_ui() {
                        ui.draw_tick(terminal_clone.clone(), rect_clone)
                            .unwrap_or_else(|err| {
                                panic!("failed to draw ui: {}", err);
                            });
                    } else {
                        panic!("widget with id {} is not a ui widget", widget_id_clone);
                    }
                    match receiver.try_recv() {
                        Ok(ControlCode::Quit) => break,
                        _ => {}
                    }
                    if start.elapsed().as_millis() < (1000 / max_refresh_rate) as u128 {
                        thread::sleep(Duration::from_millis(
                            (1000 / max_refresh_rate) as u64 - start.elapsed().as_millis() as u64,
                        ));
                    }
                }
            });
            draw_loop_thread_handles.insert(widget_id.clone(), thread_handle);
        } else {
            panic!("widget {} not found in control code channels", widget_id);
        }
        // event loop
        if let Some((_, event_receiver)) = event_channels.remove(widget_id) {
            let (_, control_code_receiver) = event_loop_control_code_channels
                .remove(widget_id)
                .unwrap_or_else(|| {
                    panic!("widget {} not found in control code channels", widget_id);
                });
            let widget_clone = pipeline
                .widgets
                .get(widget_id)
                .unwrap_or_else(|| {
                    panic!("widget {} not found in widgets", widget_id);
                })
                .clone();
            let widget_id_clone = widget_id.clone();
            let thread_handle = thread::spawn(move || {
                loop {
                    match control_code_receiver.try_recv() {
                        Ok(ControlCode::Quit) => break,
                        _ => {}
                    }
                    match event_receiver.try_recv() {
                        Ok(event) => {
                            if let Some(ui) = widget_clone.as_ui() {
                                ui.event_tick(&event).unwrap_or_else(|err| {
                                    panic!("failed to handle event: {}", err);
                                });
                            } else {
                                panic!("widget with id {} is not a ui widget", widget_id_clone);
                            }
                        }
                        _ => {}
                    }
                }
            });
            event_loop_thread_handles.insert(widget_id.clone(), thread_handle);
        } else {
            panic!("widget {} not found in event channels", widget_id);
        }
    }

    pipeline_event_loop_thread_handle.join().unwrap();
    for (_, thread_handle) in widget_tick_loop_thread_handles.into_iter() {
        match thread_handle.join() {
            Ok(_) => {}
            Err(err) => {
                panic!("widget tick loop thread panicked: {:?}", err);
            }
        }
    }
    for (_, thread_handle) in draw_loop_thread_handles.into_iter() {
        match thread_handle.join() {
            Ok(_) => {}
            Err(err) => {
                panic!("draw loop thread panicked: {:?}", err);
            }
        }
    }
    for (_, thread_handle) in event_loop_thread_handles.into_iter() {
        match thread_handle.join() {
            Ok(_) => {}
            Err(err) => {
                panic!("event loop thread panicked: {:?}", err);
            }
        }
    }
    // cleanup
    disable_raw_mode()?;
    execute!(
        terminal.lock().unwrap().backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.lock().unwrap().show_cursor()?;

    Ok(())
}
