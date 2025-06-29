use super::host_info_component::render_host_info;
use super::system_metrics_component::render_system_metrics_lines;
use crate::app::App;
use crate::app::states::SshStatus;
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
    let status_guard = futures::executor::block_on(app.ssh_statuses.lock());
    let cpu_guard = futures::executor::block_on(app.cpu_info.lock());
    let disk_guard = futures::executor::block_on(app.disk_info.lock());
    let os_guard = futures::executor::block_on(app.os_info.lock());

    let hosts = &*hosts_guard;
    let statuses = &*status_guard;
    let cpu_info = &*cpu_guard;
    let disk_info = &*disk_guard;
    let os_info = &*os_guard;

    let mut host_entries: Vec<_> = hosts.iter().collect(); // Vec<(&String, &SshHostInfo)>
    host_entries.sort_by_key(|(_, h)| &h.name);

    let total_cards = host_entries.len();
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

            let (id, info) = host_entries[idx];
            let status = statuses.get(id).unwrap_or(&SshStatus::Loading);
            let cpu = cpu_info.get(id);
            let disk = disk_info.get(id);
            let os = os_info.get(id);

            let block = Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    &info.name,
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .border_style(if app.selected_id.as_ref() == Some(id) {
                    Style::default().fg(Color::Magenta)
                } else {
                    Style::default()
                });

            let mut lines: Vec<Line> = Vec::new();
            lines.extend(render_status_lines(status));
            lines.extend(render_host_info(info));
            lines.extend(render_system_metrics_lines(info, cpu, disk, os));

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

// ─────────────────────────────────────────────────────────────
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
