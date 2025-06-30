use super::host_info_component::render_host_info;
use super::system_metrics_component::render_system_metrics_lines;
use crate::app::App;
use crate::app::AppMode;
use crate::app::states::SshStatus;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::*;

const COLUMNS: usize = 3;
const CARD_HEIGHT: u16 = 16;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Layout: Title + Overview + Grid
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Length(3), // Status overview
            Constraint::Min(0),    // Grid
        ])
        .split(area);

    if app.mode == AppMode::Search {
        let input = Paragraph::new(app.search_query.as_str())
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Search")
                    .border_style(Style::default().fg(Color::Cyan)),
            )
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(input, chunks[0]);
    } else {
        let title = Paragraph::new("SSH Hosts Overview")
            .style(Style::default().add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center);
        frame.render_widget(title, chunks[0]);
    }

    let hosts_guard = futures::executor::block_on(app.ssh_hosts.lock());
    let status_guard = futures::executor::block_on(app.ssh_statuses.lock());
    let cpu_guard = futures::executor::block_on(app.cpu_info.lock());
    let disk_guard = futures::executor::block_on(app.disk_info.lock());
    let os_guard = futures::executor::block_on(app.os_info.lock());
    let gpu_guard = futures::executor::block_on(app.gpu_info.lock());

    let hosts = &*hosts_guard;
    let statuses = &*status_guard;
    let cpu_info = &*cpu_guard;
    let disk_info = &*disk_guard;
    let os_info = &*os_guard;
    let gpu_info = &*gpu_guard;

    // Count status types
    let mut connected = 0;
    let mut loading = 0;
    let mut failed = 0;

    for status in statuses.values() {
        match status {
            SshStatus::Connected => connected += 1,
            SshStatus::Loading => loading += 1,
            SshStatus::Failed(_) => failed += 1,
        }
    }

    let overview_lines = vec![Line::from(vec![
        Span::styled("● ", Style::default().fg(Color::Green)),
        Span::raw(format!("Connected: {}  ", connected)),
        Span::styled("● ", Style::default().fg(Color::Yellow)),
        Span::raw(format!("Loading: {}  ", loading)),
        Span::styled("● ", Style::default().fg(Color::Red)),
        Span::raw(format!("Failed: {}", failed)),
    ])];

    let overview_block = Block::default()
        .borders(Borders::ALL)
        .title("Connection Summary")
        .border_style(Style::default().fg(Color::White));

    let overview = Paragraph::new(overview_lines)
        .block(overview_block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    frame.render_widget(overview, chunks[1]);

    // Grid rendering
    let grid_area = chunks[2];

    let mut host_entries: Vec<_> = hosts
        .iter()
        .filter(|(_, h)| {
            app.search_query.is_empty() || {
                let q = app.search_query.to_lowercase();
                h.name.to_lowercase().contains(&q)
                    || h.user.to_lowercase().contains(&q)
                    || h.ip.to_lowercase().contains(&q)
            }
        })
        .collect();

    host_entries.sort_by_key(|(_, h)| &h.name);

    let visible_rows = (grid_area.height / CARD_HEIGHT).max(1) as usize;
    let max_cards = visible_rows * COLUMNS;
    let display_entries = &host_entries[..host_entries.len().min(max_cards)];

    let row_constraints = vec![Constraint::Length(CARD_HEIGHT); visible_rows];
    let row_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_area);

    for (vis_row_idx, row_rect) in row_chunks.iter().enumerate() {
        let row_idx = vis_row_idx;
        if row_idx * COLUMNS >= display_entries.len() {
            continue;
        }

        let col_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(100 / COLUMNS as u16); COLUMNS])
            .split(*row_rect);

        for col in 0..COLUMNS {
            let idx = row_idx * COLUMNS + col;
            if idx >= display_entries.len() {
                continue;
            }

            let (id, info) = display_entries[idx];
            let status = statuses.get(id).unwrap_or(&SshStatus::Loading);
            let cpu = cpu_info.get(id);
            let disk = disk_info.get(id);
            let os = os_info.get(id);
            let gpu = gpu_info.get(id);

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
            lines.extend(render_system_metrics_lines(info, cpu, disk, os, gpu));

            let content = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
            frame.render_widget(content, col_chunks[col]);
        }
    }
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
