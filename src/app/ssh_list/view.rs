use crate::app::App;
use crate::app::AppMode;
use crate::app::states::SshStatus;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::*;

struct TableColors {
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    selected_row_style: Style,
    footer_border_color: Color,
}

impl TableColors {
    fn default() -> Self {
        Self {
            header_bg: Color::DarkGray,
            header_fg: Color::White,
            row_fg: Color::Gray,
            normal_row_color: Color::Black,
            alt_row_color: Color::Black,
            selected_row_style: Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Blue),
            footer_border_color: Color::Blue,
        }
    }
}

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let colors = TableColors::default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
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

    let grid_area = chunks[2];
    app.table_height = grid_area.height.saturating_sub(3) as usize;

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
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    host_entries.sort_by_key(|(_, h)| h.name.clone());
    app.visible_hosts = host_entries.clone();

    let visible_rows = grid_area.height.max(1) as usize;
    app.vertical_scroll_state = app
        .vertical_scroll_state
        .content_length(host_entries.len())
        .position(app.vertical_scroll);
    app.vertical_scroll = app
        .vertical_scroll
        .min(host_entries.len().saturating_sub(visible_rows));

    let start_index = app.vertical_scroll;
    let end_index = (start_index + visible_rows).min(host_entries.len());

    let rows = host_entries[start_index..end_index]
        .iter()
        .enumerate()
        .map(|(i, (id, info))| {
            let bg = if i % 2 == 0 {
                colors.normal_row_color
            } else {
                colors.alt_row_color
            };

            let status_str = match statuses.get(id).unwrap_or(&SshStatus::Loading) {
                SshStatus::Connected => "Connected",
                SshStatus::Loading => "Loading",
                SshStatus::Failed(_) => "Failed",
            };

            let cpu_str = match cpu_info.get(id) {
                Some(crate::app::states::CpuInfo::Success {
                    core_count,
                    usage_percent,
                }) => {
                    format!("{core_count}c, {usage_percent:.0}%")
                }
                Some(crate::app::states::CpuInfo::Failure(e)) => format!("Failed ({})", e),
                Some(crate::app::states::CpuInfo::Loading) => "Loading...".into(),
                None => "Unknown".into(),
            };

            let disk_str = match disk_info.get(id) {
                Some(crate::app::states::DiskInfo::Success { usage_percent, .. }) => {
                    format!("{:.1}%", usage_percent)
                }
                Some(crate::app::states::DiskInfo::Failure(e)) => format!("Failed ({})", e),
                Some(crate::app::states::DiskInfo::Loading) => "Loading...".into(),
                None => "Unknown".into(),
            };

            let os_str = match os_info.get(id) {
                Some(crate::app::states::OsInfo::Success { name, .. }) => name.clone(),
                Some(crate::app::states::OsInfo::Failure(e)) => format!("Failed ({})", e),
                Some(crate::app::states::OsInfo::Loading) => "Loading...".into(),
                None => "Unknown".into(),
            };

            let gpu_str = match gpu_info.get(id) {
                Some(crate::app::states::GpuInfo::Success {
                    temperature_c,
                    utilization_percent,
                    ..
                }) => {
                    format!("{temperature_c}C, {utilization_percent}%")
                }
                Some(crate::app::states::GpuInfo::Failure(e)) => format!("Failed ({})", e),
                Some(crate::app::states::GpuInfo::Loading) => "Loading...".into(),
                None => "Unknown".into(),
            };

            let user_at_host = format!("{}@{}:{}", info.user, info.ip, info.port);

            Row::new(vec![
                Cell::from(user_at_host),
                Cell::from(status_str),
                Cell::from(cpu_str),
                Cell::from(disk_str),
                Cell::from(os_str),
                Cell::from(gpu_str),
            ])
            .style(Style::default().fg(colors.row_fg).bg(bg))
            .height(2)
        });

    let header = Row::new(vec![
        Cell::from("User@Host:Port"),
        Cell::from("Status"),
        Cell::from("CPU"),
        Cell::from("Disk"),
        Cell::from("OS"),
        Cell::from("GPU"),
    ])
    .style(
        Style::default()
            .fg(colors.header_fg)
            .bg(colors.header_bg)
            .add_modifier(Modifier::BOLD),
    );

    let table = Table::new(
        rows,
        [
            Constraint::Length(40),
            Constraint::Length(16),
            Constraint::Length(16),
            Constraint::Length(16),
            Constraint::Length(16),
            Constraint::Min(16),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title("SSH Hosts"))
    .row_highlight_style(colors.selected_row_style)
    .highlight_symbol("▶ ")
    .highlight_spacing(HighlightSpacing::Always);

    frame.render_stateful_widget(table, grid_area, &mut app.table_state);

    let footer = Paragraph::new(vec![Line::from("ESC: Exit | ↑↓: Scroll | /: Search")])
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(colors.row_fg)
                .bg(colors.normal_row_color),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Controls")
                .border_style(Style::default().fg(colors.footer_border_color)),
        );

    frame.render_widget(footer, chunks[3]);
}
