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

#[tokio::main]
async fn main() -> Result<()> {
    // Exit early if required DNS backend tooling is not available.
    if !dns::DnsManager::is_available() {
        eprintln!("Error: Required DNS backend tools are not available.");
        eprintln!("{}", dns::DnsManager::availability_hint());
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    let show_help_footer = !args.iter().any(|arg| arg == "--no-help");

    // Set up the terminal UI.
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let result = run_app(&mut terminal, show_help_footer).await;

    // Restore terminal state on exit.
    disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    show_help_footer: bool,
) -> Result<()> {
    let mut app = App::new(show_help_footer)?;
    let mut last_refresh = std::time::Instant::now();
    let refresh_interval = Duration::from_secs(10);
    let mut should_redraw = true;

    while app.running {
        if should_redraw {
            terminal.draw(|frame| {
                ui::render(frame, &mut app);
            })?;
            should_redraw = false;
        }

        tokio::select! {
            // Handle periodic status refresh
            _ = tokio::time::sleep(refresh_interval.saturating_sub(last_refresh.elapsed())) => {
                app.refresh_active_dns();
                last_refresh = std::time::Instant::now();
                should_redraw = true;
            }

            // Handle keyboard input (spawned in blocking task to not freeze)
            event_res = tokio::task::spawn_blocking(|| {
                if event::poll(Duration::from_millis(100)).unwrap_or(false) {
                    return Some(event::read().unwrap());
                }
                None
            }) => {
                if let Ok(Some(Event::Key(key))) = event_res {
                    should_redraw = true;
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    // Dismiss any visible message before processing new input.
                    if app.status_message.is_some() {
                        match key.code {
                            KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => {
                                app.dismiss_message();
                                continue;
                            }
                            _ => {}
                        }
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

        if app.status_message.is_some() {
            app.check_message_timeout();
            // If timeout occurred, redraw to hide the message
            if app.status_message.is_none() {
                should_redraw = true;
            }
        }
    }

    Ok(())
}

fn handle_normal_input(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    if app.help_visible {
        match code {
            KeyCode::Char('h') | KeyCode::Esc | KeyCode::Char('q') => {
                app.close_help();
                app.help_scroll = 0;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.help_scroll = app.help_scroll.saturating_add(1);
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            _ => {}
        }
        return;
    }

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
        KeyCode::Char('h') => app.toggle_help(),
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
