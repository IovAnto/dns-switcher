use std::io;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;

mod app;
mod config;
mod dns;
mod ui;

use app::{App, AppMode};

fn main() -> Result<()> {
    // Exit early if NetworkManager is not available.
    if !dns::DnsManager::is_available() {
        eprintln!("Error: NetworkManager (nmcli) is not available.");
        eprintln!("This application requires NetworkManager to manage DNS settings.");
        std::process::exit(1);
    }

    // Set up the terminal UI.
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let result = run_app(&mut terminal);

    // Restore terminal state on exit.
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    let mut app = App::new()?;
    let mut last_refresh = std::time::Instant::now();
    let refresh_interval = Duration::from_secs(5);

    while app.running {
        // Redraw the UI every loop, refresh rates good
        terminal.draw(|frame| {
            ui::render(frame, &mut app);
        })?;

        // Periodically refresh active DNS status.
        if last_refresh.elapsed() >= refresh_interval {
            app.refresh_active_dns();
            last_refresh = std::time::Instant::now();
        }

        app.check_message_timeout();

        // Handle keyboard input.
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Dismiss any visible message before processing new input.
                if app.status_message.is_some() {
                    app.dismiss_message();
                    continue;
                }

                match app.mode {
                    AppMode::Normal => handle_normal_input(&mut app, key.code, key.modifiers),
                    AppMode::AddingCustomName | AppMode::AddingCustomIp => {
                        handle_input_mode(&mut app, key.code);
                    }
                }
            }
        }
    }

    Ok(())
}

fn handle_normal_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => app.previous(),
        KeyCode::Down | KeyCode::Char('j') => app.next(),
        KeyCode::Home | KeyCode::Char('g') => app.selected_index = 0,
        KeyCode::End | KeyCode::Char('G') => {
            if !app.providers.is_empty() {
                app.selected_index = app.providers.len() - 1;
            }
        }
        KeyCode::Enter => app.apply_selected_dns(),
        KeyCode::Char('t') => app.test_all_latencies(),
        KeyCode::Char('a') => app.start_add_custom(),
        KeyCode::Char('d') | KeyCode::Delete => app.delete_selected(),
        KeyCode::Char('r') => app.reset_dns(),
        KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.quit(),
        KeyCode::Esc => app.quit(),
        _ => {}
    }
}

fn handle_input_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char(c) => app.handle_input_char(c),
        KeyCode::Backspace => app.handle_input_backspace(),
        KeyCode::Enter => app.confirm_input(),
        KeyCode::Esc => app.cancel_input(),
        _ => {}
    }
}