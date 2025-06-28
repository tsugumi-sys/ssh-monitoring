use crate::app::App;
use crate::app::states::SshStatus;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::*;

const COLUMNS: usize = 3;
const CARD_HEIGHT: u16 = 8;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Layout: Title + Grid
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    // Title
    let title = Paragraph::new("SSH Hosts Overview (j/k to scroll)")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    let grid_area = chunks[1];

    // Load SSH hosts list
    let hosts_guard = futures::executor::block_on(app.ssh_hosts.lock());
    let hosts = &*hosts_guard;

    let total_cards = hosts.len();
    let total_rows = (total_cards + COLUMNS - 1) / COLUMNS;
    let visible_rows = (grid_area.height / CARD_HEIGHT).max(1) as usize;

    let scroll_offset = app
        .scroll_offset
        .min(total_rows.saturating_sub(visible_rows));

    // Layout rows
    let row_constraints = vec![Constraint::Length(CARD_HEIGHT); visible_rows];
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_area);

    for (vis_row_idx, row_rect) in row_chunks.iter().enumerate() {
        let row_idx = scroll_offset + vis_row_idx;
        if row_idx >= total_rows {
            continue;
        }

        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100 / COLUMNS as u16); COLUMNS])
            .split(*row_rect);

        for col in 0..COLUMNS {
            let idx = row_idx * COLUMNS + col;
            if idx >= total_cards {
                continue;
            }

            let host_state = &hosts[idx];

            let status_span = match &host_state.status {
                SshStatus::Connected => {
                    Span::styled("✅ Connected", Style::default().fg(Color::Green))
                }
                SshStatus::Failed(err) => Span::styled(
                    format!("❌ Failed: {}", err),
                    Style::default().fg(Color::Red),
                ),
                SshStatus::Loading => {
                    Span::styled("⏳ Loading", Style::default().fg(Color::Yellow))
                }
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    &host_state.info.name,
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .border_style(if idx == app.selected_index {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default()
                });

            let mut lines = vec![
                Line::from(status_span),
                Line::from(format!(
                    "{}@{}:{}",
                    host_state.info.user, host_state.info.ip, host_state.info.port
                )),
                Line::from(format!("🔑 Key: {}", host_state.info.identity_file)),
            ];

            if host_state.info.is_placeholder_identity_file() {
                lines.push(Line::from(Span::styled(
                    "⚠️ No IdentityFile set. Monitoring disabled.",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                )));
            } else {
                lines.extend([
                    Line::from("🖥️  CPU: 23%"),
                    Line::from("🧠 Mem: 2.3G / 8G"),
                    Line::from("🎮 GPU: 2GB"),
                    Line::from("💾 Storage: 40G / 100G"),
                ]);
            }

            let content = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });

            frame.render_widget(content, col_chunks[col]);
        }
    }

    // Scrollbar
    let mut scrollbar_state = ScrollbarState::new(total_rows).position(scroll_offset);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        grid_area,
        &mut scrollbar_state,
    );
}
