use crate::app::App;
use crate::app::states::{SshHostState, SshStatus};
use crate::app::tasks::cpu_status_task::CpuInfoStatus;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::*;

const COLUMNS: usize = 3;
const CARD_HEIGHT: u16 = 16;

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

    let hosts_guard = futures::executor::block_on(app.ssh_hosts.lock());
    let cpu_guard = futures::executor::block_on(app.cpu_statuses.lock());

    let hosts = &*hosts_guard;
    let cpu_statuses = &*cpu_guard;

    let total_cards = hosts.len();
    let total_rows = total_cards.div_ceil(COLUMNS);
    let visible_rows = (grid_area.height / CARD_HEIGHT).max(1) as usize;

    let scroll_offset = app
        .scroll_offset
        .min(total_rows.saturating_sub(visible_rows));

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
            let cpu_status = cpu_statuses.get(idx);

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

            let mut lines: Vec<Line> = Vec::new();
            lines.extend(render_status_lines(&host_state.status));
            lines.extend(render_host_info_lines(host_state));
            lines.extend(render_system_metrics_lines(host_state, cpu_status));

            let content = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
            frame.render_widget(content, col_chunks[col]);
        }
    }

    let mut scrollbar_state = ScrollbarState::new(total_rows).position(scroll_offset);
    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
        grid_area,
        &mut scrollbar_state,
    );
}

// Sub-widgets

fn render_status_lines(status: &SshStatus) -> Vec<Line<'_>> {
    let status_span = match status {
        SshStatus::Connected => Span::styled("● Connected", Style::default().fg(Color::Green)),
        SshStatus::Loading => Span::styled("● Loading...", Style::default().fg(Color::Yellow)),
        SshStatus::Failed(err) => Span::styled(
            format!("● Failed: {}", err),
            Style::default().fg(Color::Red),
        ),
    };

    vec![
        Line::from(vec![
            Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
            status_span,
        ]),
        Line::raw(""),
    ]
}

fn render_host_info_lines(host: &SshHostState) -> Vec<Line<'_>> {
    let mut lines = vec![
        Line::from(Span::styled(
            "Host Info",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(format!("User: {}", host.info.user)),
        Line::from(format!("IP:   {}", host.info.ip)),
        Line::from(format!("Port: {}", host.info.port)),
        Line::from(format!("Key:  {}", host.info.identity_file)),
    ];

    if host.info.is_placeholder_identity_file() {
        lines.push(Line::from(Span::styled(
            "Warning: No IdentityFile set. Monitoring disabled.",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    lines.push(Line::raw(""));
    lines
}

fn render_system_metrics_lines<'a>(
    host: &'a SshHostState,
    cpu_status: Option<&'a CpuInfoStatus>,
) -> Vec<Line<'a>> {
    let mut lines = vec![Line::from(Span::styled(
        "System Metrics",
        Style::default().add_modifier(Modifier::UNDERLINED),
    ))];

    if host.info.is_placeholder_identity_file() {
        lines.push(Line::from(Span::styled(
            "Not available (no identity file set)",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
    } else {
        match cpu_status {
            Some(CpuInfoStatus::Loading) => {
                lines.push(Line::from("CPU: Loading..."));
            }
            Some(CpuInfoStatus::Fetched(info)) => {
                lines.push(Line::from(format!(
                    "CPU: {} cores, {:.1}% usage",
                    info.core_count, info.usage_percent
                )));
            }
            Some(CpuInfoStatus::Failed(e)) => {
                lines.push(Line::from(Span::styled(
                    format!("CPU: Failed - {}", e),
                    Style::default().fg(Color::Red),
                )));
            }
            None => {
                lines.push(Line::from("CPU: Unknown"));
            }
        }
    }

    lines
}
