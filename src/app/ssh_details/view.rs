use crate::app::App;
use crate::app::states::{CpuInfo, DiskInfo, GpuInfo, OsInfo, SshStatus};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let hosts_guard = futures::executor::block_on(app.ssh_hosts.lock());
    let status_guard = futures::executor::block_on(app.ssh_statuses.lock());
    let cpu_guard = futures::executor::block_on(app.cpu_info.lock());
    let disk_guard = futures::executor::block_on(app.disk_info.lock());
    let os_guard = futures::executor::block_on(app.os_info.lock());
    let gpu_guard = futures::executor::block_on(app.gpu_info.lock());

    let host = app.selected_id.as_ref().and_then(|id| hosts_guard.get(id));

    let host_name = host
        .map(|h| h.name.clone())
        .unwrap_or_else(|| "<none>".to_string());

    let status = app
        .selected_id
        .as_ref()
        .and_then(|id| status_guard.get(id))
        .cloned()
        .unwrap_or(SshStatus::Loading);

    let cpu = app.selected_id.as_ref().and_then(|id| cpu_guard.get(id));
    let disk = app.selected_id.as_ref().and_then(|id| disk_guard.get(id));
    let os = app.selected_id.as_ref().and_then(|id| os_guard.get(id));
    let gpu = app.selected_id.as_ref().and_then(|id| gpu_guard.get(id));

    let (status_text, status_style, status_msg) = match &status {
        SshStatus::Connected => (
            "🟢 Connected".to_string(),
            Style::default().fg(Color::Green),
            None,
        ),
        SshStatus::Loading => (
            "🟡 Loading".to_string(),
            Style::default().fg(Color::Yellow),
            None,
        ),
        SshStatus::Failed(msg) => (
            "🔴 Failed".to_string(),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            Some(msg.clone()),
        ),
    };

    let header_lines: Vec<Line> = match status_msg {
        Some(msg) => vec![
            Line::raw(format!("Host: {} | Status: {}", host_name, status_text)),
            Line::styled(msg, Style::default().fg(Color::Red)),
            Line::raw("Press 'q' to go back"),
        ],
        None => vec![Line::raw(format!(
            "Host: {} | Status: {} | Press 'q' to go back",
            host_name, status_text
        ))],
    };

    let header = Paragraph::new(header_lines)
        .style(status_style)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(header, chunks[0]);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // CPU INFO BLOCK
    let cpu_lines: Vec<Line> = match cpu {
        Some(CpuInfo::Success {
            core_count,
            usage_percent,
        }) => vec![
            Line::raw(format!("Cores: {core_count}")),
            Line::raw(format!("Usage: {usage_percent:.1}%")),
        ],
        Some(CpuInfo::Failure(e)) => vec![Line::styled(
            format!("Error: {e}"),
            Style::default().fg(Color::Red),
        )],
        Some(CpuInfo::Loading) => vec![Line::raw("Loading...")],
        None => vec![Line::raw("N/A")],
    };

    let cpu_block = Paragraph::new(cpu_lines)
        .block(Block::default().borders(Borders::ALL).title("🧠 CPU Usage"));
    frame.render_widget(cpu_block, top_chunks[0]);

    // MEMORY BLOCK - dummy data
    let mem_lines = vec![Line::raw("Total: 16 GiB"), Line::raw("Used: 8.3 GiB (52%)")];
    let mem_block = Paragraph::new(mem_lines)
        .block(Block::default().borders(Borders::ALL).title("Memory Usage"));
    frame.render_widget(mem_block, top_chunks[1]);

    // GPU INFO
    let gpu_lines: Vec<Line> = match gpu {
        Some(GpuInfo::Success {
            name,
            memory_total_mb,
            memory_used_mb,
            utilization_percent,
            temperature_c,
        }) => vec![
            Line::raw(format!("GPU: {}", name)),
            Line::raw(format!("Util: {}%", utilization_percent)),
            Line::raw(format!("Mem: {}/{}MB", memory_used_mb, memory_total_mb)),
            Line::raw(format!("{}°C", temperature_c)),
        ],
        Some(GpuInfo::Failure(e)) => {
            if e == "nvidia-smi not available" {
                vec![Line::raw("N/A")]
            } else {
                vec![Line::styled(
                    format!("Error: {e}"),
                    Style::default().fg(Color::Red),
                )]
            }
        }
        Some(GpuInfo::Loading) => vec![Line::raw("Loading...")],
        None => vec![Line::raw("N/A")],
    };
    let gpu_block =
        Paragraph::new(gpu_lines).block(Block::default().borders(Borders::ALL).title("🔧 GPU"));
    frame.render_widget(gpu_block, chunks[2]);

    // DISK INFO
    let disk_lines: Vec<Line> = match disk {
        Some(DiskInfo::Success {
            total,
            used,
            avail,
            usage_percent,
        }) => vec![
            Line::raw(format!("Total: {}", total)),
            Line::raw(format!("Used: {}", used)),
            Line::raw(format!("Avail: {}", avail)),
            Line::raw(format!("Usage: {}", usage_percent)),
        ],
        Some(DiskInfo::Failure(e)) => vec![Line::styled(
            format!("Error: {e}"),
            Style::default().fg(Color::Red),
        )],
        Some(DiskInfo::Loading) => vec![Line::raw("Loading...")],
        None => vec![Line::raw("N/A")],
    };
    let disk_block = Paragraph::new(disk_lines)
        .block(Block::default().borders(Borders::ALL).title("Disk Usage"));
    frame.render_widget(disk_block, chunks[3]);

    // OS INFO
    let os_lines: Vec<Line> = match os {
        Some(OsInfo::Success {
            name,
            version,
            timezone,
        }) => vec![
            Line::raw(format!("{} {}", name, version)),
            Line::raw(format!("TZ: {}", timezone)),
        ],
        Some(OsInfo::Failure(e)) => vec![Line::styled(
            format!("Error: {e}"),
            Style::default().fg(Color::Red),
        )],
        Some(OsInfo::Loading) => vec![Line::raw("Loading...")],
        None => vec![Line::raw("N/A")],
    };
    let os_block =
        Paragraph::new(os_lines).block(Block::default().borders(Borders::ALL).title("OS Info"));
    frame.render_widget(os_block, chunks[4]);

    // Dummy top process tables
    let proc_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[5]);

    let cpu_header = Row::new(vec!["PID", "USER", "%CPU", "MEM", "COMMAND"])
        .style(Style::default().add_modifier(Modifier::BOLD));
    let cpu_rows = vec![
        Row::new(vec![
            "4231",
            "postgres",
            "31.2",
            "512MB",
            "/usr/lib/postgres",
        ]),
        Row::new(vec!["1984", "root", "21.5", "128MB", "/usr/bin/containerd"]),
    ];
    let cpu_table = Table::new(
        cpu_rows,
        [
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(10),
        ],
    )
    .header(cpu_header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("🔝 Top CPU Processes"),
    );
    frame.render_widget(cpu_table, proc_chunks[0]);

    let mem_header = Row::new(vec!["PID", "USER", "%MEM", "CPU", "COMMAND"])
        .style(Style::default().add_modifier(Modifier::BOLD));
    let mem_rows = vec![
        Row::new(vec!["1561", "chrome", "19.2", "8.3%", "/opt/chrome/chrome"]),
        Row::new(vec![
            "987",
            "redis",
            "12.3",
            "3.1%",
            "/usr/bin/redis-server",
        ]),
    ];
    let mem_table = Table::new(
        mem_rows,
        [
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Min(10),
        ],
    )
    .header(mem_header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("🧠 Top Memory Processes"),
    );
    frame.render_widget(mem_table, proc_chunks[1]);
}
