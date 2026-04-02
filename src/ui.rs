use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph},
};

use crate::app::{App, AppMode};

mod colors {
    use ratatui::style::Color;

    pub const PRIMARY: Color = Color::Cyan;
    pub const ACTIVE: Color = Color::Green;
    pub const ERROR: Color = Color::Red;
    pub const MUTED: Color = Color::DarkGray;
    pub const HIGHLIGHT_BG: Color = Color::Rgb(40, 40, 60);
}

pub fn render(frame: &mut Frame, app: &mut App) {
    // Main screen layout: header, content, footer.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_provider_list(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    // Show modal input when adding a custom DNS entry.
    if matches!(
        app.mode,
        AppMode::AddingCustomName | AppMode::AddingCustomIp
    ) {
        render_input_popup(frame, app);
    }

    // Toast notifications appear above the main layout.
    if app.status_message.is_some() {
        render_status_toast(frame, app);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let active_dns_text = match &app.active_dns {
        Some(ip) => {
            let name = app
                .providers
                .iter()
                .find(|p| p.primary == ip || p.secondary == ip)
                .map(|p| p.name.to_string())
                .unwrap_or_else(|| "Custom".to_string());
            format!(" Active: {} ({}) ", name, ip)
        }
        None => " Active: Checking... ".to_string(),
    };

    let header = Paragraph::new(Line::from(vec![
        Span::styled("DNS Switcher", Style::default().fg(colors::PRIMARY).bold()),
        Span::raw("  │  "),
        Span::styled(active_dns_text, Style::default().fg(colors::ACTIVE)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors::PRIMARY)),
    );

    frame.render_widget(header, area);
}

fn render_provider_list(frame: &mut Frame, app: &mut App, area: Rect) {
    // Split list and details into two columns.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let items: Vec<ListItem> = app
        .providers
        .iter()
        .map(|provider| {
            let is_active = app
                .active_dns
                .as_ref()
                .map(|ip| ip == provider.primary || ip == provider.secondary)
                .unwrap_or(false);

            let indicator = if is_active { "●" } else { "○" };

            let latency = app
                .latencies
                .get(provider.id)
                .map(|ms| format!(" {:>3}ms", ms))
                .unwrap_or_else(|| "     ".to_string());

            let style = if is_active {
                Style::default().fg(colors::ACTIVE).bold()
            } else {
                Style::default()
            };

            let custom_tag = if provider.is_custom { " [custom]" } else { "" };

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", indicator), style),
                Span::styled(format!("{:<12}{}", provider.name, custom_tag), style),
                Span::styled(latency, Style::default().fg(colors::MUTED)),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Providers ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY))
                .padding(Padding::vertical(1)),
        )
        .highlight_style(
            Style::default()
                .bg(colors::HIGHLIGHT_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    // Show details for the currently selected provider.
    if let Some(provider) = app.selected_provider() {
        let latency_text = app
            .latencies
            .get(provider.id)
            .map(|ms| format!("{} ms", ms))
            .unwrap_or_else(|| "Not tested".to_string());

        let details = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(colors::MUTED)),
                Span::styled(provider.name, Style::default().bold()),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Primary: ", Style::default().fg(colors::MUTED)),
                Span::raw(provider.primary),
            ]),
            Line::from(vec![
                Span::styled("Secondary: ", Style::default().fg(colors::MUTED)),
                Span::raw(provider.secondary),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Latency: ", Style::default().fg(colors::MUTED)),
                Span::raw(latency_text),
            ]),
        ];

        let details_widget = Paragraph::new(details).block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY))
                .padding(Padding::new(2, 2, 1, 1)),
        );

        frame.render_widget(details_widget, chunks[1]);
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    // Contextual help changes while loading.
    let commands = if app.is_loading {
        vec![
            Span::styled(" ⏳ ", Style::default().fg(colors::PRIMARY)),
            Span::raw("Processing..."),
        ]
    } else {
        vec![
            Span::styled(" ↑↓ ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Navigate  "),
            Span::styled(" Enter ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Apply  "),
            Span::styled(" t ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Test speed  "),
            Span::styled(" a ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Add custom  "),
            Span::styled(" d ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Delete  "),
            Span::styled(" r ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Reset ISP  "),
            Span::styled(" q ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Quit"),
        ]
    };

    let footer = Paragraph::new(Line::from(commands)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors::MUTED)),
    );

    frame.render_widget(footer, area);
}

fn render_input_popup(frame: &mut Frame, app: &App) {
    // Centered modal dialog for custom DNS input.
    let area = centered_rect(50, 20, frame.area());

    frame.render_widget(ratatui::widgets::Clear, area);

    let (title, placeholder) = match app.mode {
        AppMode::AddingCustomName => ("Add Custom DNS - Name", "e.g. My DNS"),
        AppMode::AddingCustomIp => ("Add Custom DNS - IP Address", "e.g. 1.2.3.4 5.6.7.8"),
        _ => ("", ""),
    };

    let input_text = if app.input_buffer.is_empty() {
        Span::styled(placeholder, Style::default().fg(colors::MUTED))
    } else {
        Span::raw(&app.input_buffer)
    };

    let popup = Paragraph::new(vec![
        Line::from(""),
        Line::from(input_text),
        Line::from(""),
        Line::from(Span::styled(
            "Press Enter to confirm, Esc to cancel",
            Style::default().fg(colors::MUTED),
        )),
    ])
    .block(
        Block::default()
            .title(format!(" {} ", title))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors::PRIMARY))
            .padding(Padding::horizontal(2)),
    );

    frame.render_widget(popup, area);

    let cursor_x = area.x + 3 + app.input_buffer.len() as u16;
    let cursor_y = area.y + 2;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn render_status_toast(frame: &mut Frame, app: &App) {
    if let Some(msg) = &app.status_message {
        let color = if app.is_error {
            colors::ERROR
        } else {
            colors::ACTIVE
        };
        let icon = if app.is_error { "✗" } else { "✓" };

        let msg_width = (msg.len() + 6).clamp(20, 50) as u16;
        let toast_height = 3u16;

        let screen = frame.area();
        let area = Rect {
            x: screen.width.saturating_sub(msg_width + 2),
            y: 1,
            width: msg_width,
            height: toast_height,
        };

        frame.render_widget(ratatui::widgets::Clear, area);

        let toast = Paragraph::new(Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color).bold()),
            Span::styled(msg.as_str(), Style::default().fg(color)),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(color)),
        )
        .alignment(Alignment::Left);

        frame.render_widget(toast, area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    // Build a centered rectangle using vertical then horizontal splits.
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
