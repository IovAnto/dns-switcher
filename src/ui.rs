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

const BREAKPOINT_NARROW: u16 = 60;
const BREAKPOINT_TINY_W: u16 = 40;
const BREAKPOINT_TINY_H: u16 = 8;
const BREAKPOINT_COMPACT_HEADER: u16 = 65;
const BREAKPOINT_STACKED_TOAST: u16 = 60;

pub fn render(frame: &mut Frame, app: &mut App) {
    // Main screen layout: header, content, footer.
    let screen = frame.area();
    let compact_height = screen.height < 30;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(if compact_height { 0 } else { 1 })
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(if screen.height < 12 { 2 } else { 3 }),
        ])
        .split(screen);

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
        render_status_toast(frame, app, chunks[0], chunks[1]);
    }

    if app.help_visible {
        render_help_popup(frame, app);
    }
}

fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let compact = area.width < BREAKPOINT_COMPACT_HEADER;
    let (active_name, active_ip) = match &app.active_dns {
        Some(ip) => {
            let name = app
                .providers
                .iter()
                .find(|p| p.primary == ip || p.secondary == ip)
                .map(|p| p.name.to_string())
                .unwrap_or_else(|| "Custom".to_string());
            (name, ip.clone())
        }
        None => ("Checking...".to_string(), "...".to_string()),
    };

    let status = if app.is_loading { "(Updating...) " } else { "" };

    let header_content = if compact {
        vec![
            Span::styled("  DNS Switcher", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(" │ "),
            Span::styled(status, Style::default().fg(colors::PRIMARY)),
            Span::styled(active_ip, Style::default().fg(colors::ACTIVE).bold()),
        ]
    } else {
        vec![
            Span::styled("  DNS Switcher", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(" │ "),
            Span::styled(
                if status.is_empty() { " " } else { status },
                Style::default()
                    .fg(colors::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Active: ", Style::default().fg(colors::MUTED)),
            Span::styled(
                format!("{} ({})", active_name, active_ip),
                Style::default().fg(colors::ACTIVE).bold(),
            ),
        ]
    };

    frame.render_widget(
        Paragraph::new(Line::from(header_content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY)),
        ),
        area,
    );
}

fn render_provider_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let narrow = area.width < BREAKPOINT_NARROW;
    let tiny = area.width < BREAKPOINT_TINY_W || area.height < BREAKPOINT_TINY_H;
    let estimated_list_width = if narrow { area.width } else { area.width / 2 };
    let content_budget = estimated_list_width.saturating_sub(10) as usize;

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
                .map(|ms| format!(" {}ms", ms))
                .unwrap_or_default();

            let show_latency = content_budget >= 14;

            let style = if is_active {
                Style::default().fg(colors::ACTIVE).bold()
            } else {
                Style::default()
            };

            let custom_tag = if provider.is_custom {
                if content_budget < 16 {
                    " [c]"
                } else {
                    " [custom]"
                }
            } else {
                ""
            };

            let base_label = format!("{}{}", provider.name, custom_tag);

            let label = truncate_with_ellipsis(&base_label, content_budget.max(4));

            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", indicator), style),
                Span::styled(label, style),
                Span::styled(
                    if show_latency { latency } else { String::new() },
                    Style::default().fg(colors::MUTED),
                ),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Providers ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY))
                .padding(Padding::vertical(if narrow { 0 } else { 1 })),
        )
        .highlight_style(
            Style::default()
                .bg(colors::HIGHLIGHT_BG)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_index));

    if tiny {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);

        frame.render_stateful_widget(list, rows[0], &mut list_state);
        render_provider_details_strip(frame, app, rows[1]);
        return;
    }

    if narrow {
        // Keep details visible even on tight layouts.
        let details_height = if area.height >= 16 { 7 } else { 5 };

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(details_height)])
            .split(area);

        frame.render_stateful_widget(list, rows[0], &mut list_state);
        render_provider_details(frame, app, rows[1], true);
        return;
    }

    // Split list and details into two columns.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    render_provider_details(frame, app, chunks[1], false);
}

fn truncate_with_ellipsis(input: &str, max_chars: usize) -> String {
    if max_chars <= 1 {
        return "…".to_string();
    }

    let count = input.chars().count();
    if count <= max_chars {
        return input.to_string();
    }

    let kept: String = input.chars().take(max_chars - 1).collect();
    format!("{}…", kept)
}

fn render_provider_details(frame: &mut Frame, app: &App, area: Rect, compact: bool) {
    // Show details for the currently selected provider.
    if let Some(provider) = app.selected_provider() {
        let latency_text = app
            .latencies
            .get(provider.id)
            .map(|ms| format!("{} ms", ms))
            .unwrap_or_else(|| "Not tested".to_string());

        let ultra_compact = compact && (area.height <= 4 || area.width < 48);

        let details = if ultra_compact {
            vec![Line::from(vec![
                Span::styled(provider.name, Style::default().bold()),
                Span::raw("  "),
                Span::styled(provider.primary, Style::default()),
            ])]
        } else if compact {
            vec![Line::from(vec![
                Span::styled(provider.name, Style::default().bold()),
                Span::styled("  ", Style::default()),
                Span::styled(provider.primary, Style::default()),
                Span::styled("  ", Style::default()),
                Span::styled(latency_text, Style::default().fg(colors::MUTED)),
            ])]
        } else {
            let mut details = vec![
                Line::from(vec![
                    Span::styled("Name: ", Style::default().fg(colors::MUTED)),
                    Span::styled(provider.name, Style::default().bold()),
                ]),
                Line::from(""),
            ];

                details.push(Line::from(vec![
                    Span::styled("Primary: ", Style::default().fg(colors::MUTED)),
                    Span::raw(provider.primary),
                ]));
                details.push(Line::from(vec![
                    Span::styled("Secondary: ", Style::default().fg(colors::MUTED)),
                    Span::raw(provider.secondary),
                ]));

            details.push(Line::from(""));
            details.push(Line::from(vec![
                Span::styled("Latency: ", Style::default().fg(colors::MUTED)),
                Span::raw(latency_text),
            ]));

            details
        };

        let details_widget = Paragraph::new(details).block(
            Block::default()
                .title(" Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY))
                .padding(Padding::new(
                    if ultra_compact {
                        0
                    } else if compact {
                        1
                    } else {
                        2
                    },
                    1,
                    1,
                    1,
                )),
        );

        frame.render_widget(details_widget, area);
    }
}

fn render_provider_details_strip(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(provider) = app.selected_provider() {
        let latency_text = app
            .latencies
            .get(provider.id)
            .map(|ms| format!("{}ms", ms))
            .unwrap_or_else(|| "n/a".to_string());

        let line = Line::from(vec![
            Span::styled("Details: ", Style::default().fg(colors::MUTED)),
            Span::styled(provider.name, Style::default().bold()),
            Span::raw(" "),
            Span::styled(provider.primary, Style::default().fg(colors::ACTIVE)),
            Span::raw("  "),
            Span::styled(latency_text, Style::default().fg(colors::MUTED)),
        ]);

        let strip = Paragraph::new(line).alignment(Alignment::Left);
        frame.render_widget(strip, area);
    }
}

fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let content = if app.is_loading {
        vec![
            Span::styled(" ⏳ ", Style::default().fg(colors::PRIMARY)),
            Span::raw("Processing..."),
        ]
    } else if !app.show_help_footer {
        if let Some(provider) = app.selected_provider() {
            let (latency_text, quality_text, quality_color) = match app.latencies.get(provider.id) {
                Some(&ms) => {
                    let (label, color) = match ms {
                        0..=35 => ("excelent", Color::Green),
                        36..=75 => ("good", Color::Cyan),
                        76..=150 => ("stable", Color::Yellow),
                        _ => ("slow", Color::Red),
                    };
                    (format!("{}ms", ms), label, color)
                }
                None => ("N/A".to_string(), "NO TEST", colors::MUTED),
            };

            let provider_type = if provider.is_custom {
                "Custom"
            } else {
                "System"
            };

            let mut info = vec![
                Span::styled(
                    " DNS Provider:",
                    Style::default().fg(colors::PRIMARY).bold(),
                ),
                Span::raw(" │ "),
                Span::styled("Pos: ", Style::default().fg(colors::MUTED)),
                Span::raw(format!("{}/{}", app.selected_index + 1, app.providers.len())),
                Span::raw(" │ "),
                Span::styled("Type: ", Style::default().fg(colors::MUTED)),
                Span::raw(provider_type),
                Span::raw(" │ "),
                Span::styled("Speed: ", Style::default().fg(colors::MUTED)),
                Span::styled(latency_text, Style::default().fg(quality_color).bold()),
                Span::raw(" ["),
                Span::styled(quality_text, Style::default().fg(quality_color).bold()),
                Span::raw("]"),
            ];

            if area.width > 95 {
                info.push(Span::raw(" │ "));
                info.push(Span::styled(
                    " h ",
                    Style::default().fg(colors::PRIMARY).bold(),
                ));
                info.push(Span::raw("Help  "));
                info.push(Span::styled(
                    " q ",
                    Style::default().fg(colors::PRIMARY).bold(),
                ));
                info.push(Span::raw("Quit"));
            }

            info
        } else {
            vec![
                Span::styled(
                    " No provider selected ",
                    Style::default().fg(colors::MUTED),
                ),
                Span::raw(" │ "),
                Span::styled(" h ", Style::default().fg(colors::PRIMARY).bold()),
                Span::raw("Help"),
            ]
        }
    } else if area.width < 56 {
        vec![
            Span::styled(" ↑↓ ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Nav  "),
            Span::styled(" Enter ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Apply  "),
            Span::styled(" h ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Help  "),
            Span::styled(" q ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Quit"),
        ]
    } else if area.width < 90 {
        vec![
            Span::styled(" ↑↓ ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Navigate  "),
            Span::styled(" Enter ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Apply  "),
            Span::styled(" t ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Test  "),
            Span::styled(" a ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Add  "),
            Span::styled(" h ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Help  "),
            Span::styled(" q ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Quit"),
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
            Span::styled(" h ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Help  "),
            Span::styled(" q ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw("Quit"),
        ]
    };

    frame.render_widget(
        Paragraph::new(Line::from(content)).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::MUTED)),
        ),
        area,
    );
}

fn render_input_popup(frame: &mut Frame, app: &App) {
    // Centered modal dialog for custom DNS input.
    let screen = frame.area();
    let area = centered_rect(
        if screen.width < 90 { 80 } else { 50 },
        if screen.height < 30 { 30 } else { 20 },
        screen,
    );

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

    let max_cursor_x = area.x + area.width.saturating_sub(2);
    let cursor_x = (area.x + 3 + app.input_buffer.len() as u16).min(max_cursor_x);
    let cursor_y = area.y + 2;
    frame.set_cursor_position((cursor_x, cursor_y));
}

fn render_status_toast(frame: &mut Frame, app: &App, header_area: Rect, content_area: Rect) {
    if let Some(msg) = &app.status_message {
        let color = if app.is_error {
            colors::ERROR
        } else {
            colors::ACTIVE
        };
        let icon = if app.is_error { "✗" } else { "✓" };

        if header_area.width < 20 || header_area.height < 3 {
            return;
        }

        let list_is_stacked = content_area.width < BREAKPOINT_STACKED_TOAST;

        let (anchor_x, target_width) = if list_is_stacked {
            (header_area.x + 1, header_area.width.saturating_sub(2))
        } else {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(content_area);
            (cols[1].x, cols[1].width)
        };

        let right_limit = (header_area.x + header_area.width).saturating_sub(1);
        let available_w = right_limit
            .saturating_sub(anchor_x)
            .saturating_add(1)
            .max(10);
        let natural_w = ((msg.chars().count() + 6) as u16).max(10);
        let msg_width = natural_w.min(target_width).min(available_w);

        let column_right = anchor_x.saturating_add(target_width.saturating_sub(1));
        let align_right = column_right.saturating_sub(msg_width).saturating_add(1);
        let max_x = right_limit.saturating_sub(msg_width).saturating_add(1);

        let area = Rect {
            x: align_right.min(max_x),
            y: header_area.y,
            width: msg_width,
            height: 3,
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

fn render_help_popup(frame: &mut Frame, app: &App) {
    let screen = frame.area();
    let area = centered_rect(
        if screen.width < 60 { 95 } else if screen.width < 90 { 80 } else { 60 },
        if screen.height < 15 { 95 } else if screen.height < 30 { 80 } else { 50 },
        screen,
    );

    frame.render_widget(ratatui::widgets::Clear, area);

    let help_lines = vec![
        Line::from(vec![
            Span::styled("Navigation    ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": ↑/↓, j/k, Home/End"),
        ]),
        Line::from(vec![
            Span::styled("Apply DNS     ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": Enter"),
        ]),
        Line::from(vec![
            Span::styled("Test latency  ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": t"),
        ]),
        Line::from(vec![
            Span::styled("Add custom    ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": a"),
        ]),
        Line::from(vec![
            Span::styled("Delete custom ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": d / Delete"),
        ]),
        Line::from(vec![
            Span::styled("Reset DNS     ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": r"),
        ]),
        Line::from(vec![
            Span::styled("Help          ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": h (toggle)"),
        ]),
        Line::from(vec![
            Span::styled("Quit          ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": q / Esc / Ctrl+C"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Scroll Help   ", Style::default().fg(colors::PRIMARY).bold()),
            Span::raw(": ↑/↓ or j/k"),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Press h, Esc or q to close",
            Style::default().fg(colors::MUTED).italic(),
        )),
    ];

    let popup = Paragraph::new(help_lines)
        .scroll((app.help_scroll, 0))
        .block(
            Block::default()
                .title(" Help (↑/↓ to scroll) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors::PRIMARY))
                .padding(Padding::new(if area.width < 40 { 1 } else { 2 }, 1, 1, 1)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(popup, area);
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
