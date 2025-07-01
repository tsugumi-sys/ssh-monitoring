use super::table_theme::TableColors;
use crate::app::states::{CpuInfo, DiskInfo, GpuInfo, OsInfo, SshHostInfo, SshStatus};
use ratatui::prelude::*;
use ratatui::text::Span;
use ratatui::widgets::*;

#[allow(clippy::too_many_arguments)]
pub fn render_host_row(
    i: usize,
    info: &SshHostInfo,
    status: &SshStatus,
    cpu: Option<&CpuInfo>,
    disk: Option<&DiskInfo>,
    os: Option<&OsInfo>,
    gpu: Option<&GpuInfo>,
    colors: &TableColors,
) -> Row<'static> {
    let bg = if i % 2 == 0 {
        colors.normal_row_color
    } else {
        colors.alt_row_color
    };

    let user_at_host = format!("{}@{}:{}", info.user, info.ip, info.port);

    let status_cell = match status {
        SshStatus::Connected => {
            Cell::from(Span::styled("Connected", Style::default().fg(Color::Green)))
        }
        SshStatus::Loading => {
            Cell::from(Span::styled("Loading", Style::default().fg(Color::Yellow)))
        }
        SshStatus::Failed(_) => Cell::from(Span::styled("Failed", Style::default().fg(Color::Red))),
    };

    let cpu_cell = match cpu {
        Some(CpuInfo::Success {
            core_count,
            usage_percent,
        }) => Cell::from(Span::styled(
            format!("{core_count}c, {usage_percent:.0}%"),
            Style::default().fg(Color::White),
        )),
        Some(CpuInfo::Failure(_)) => {
            Cell::from(Span::styled("Failed", Style::default().fg(Color::Red)))
        }
        Some(CpuInfo::Loading) => Cell::from(Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )),
        None => Cell::from("Unknown"),
    };

    let disk_cell = match disk {
        Some(DiskInfo::Success { usage_percent, .. }) => Cell::from(Span::styled(
            format!("{:.1}%", usage_percent),
            Style::default().fg(Color::White),
        )),
        Some(DiskInfo::Failure(_)) => {
            Cell::from(Span::styled("Failed", Style::default().fg(Color::Red)))
        }
        Some(DiskInfo::Loading) => Cell::from(Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )),
        None => Cell::from("Unknown"),
    };

    let os_cell = match os {
        Some(OsInfo::Success { name, .. }) => Cell::from(Span::styled(
            name.clone(),
            Style::default().fg(Color::White),
        )),
        Some(OsInfo::Failure(_)) => {
            Cell::from(Span::styled("Failed", Style::default().fg(Color::Red)))
        }
        Some(OsInfo::Loading) => Cell::from(Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )),
        None => Cell::from("Unknown"),
    };

    let gpu_cell = match gpu {
        Some(GpuInfo::Success {
            temperature_c,
            utilization_percent,
            ..
        }) => Cell::from(Span::styled(
            format!("{temperature_c}C, {utilization_percent}%"),
            Style::default().fg(Color::White),
        )),
        Some(GpuInfo::Failure(_)) => {
            Cell::from(Span::styled("Failed", Style::default().fg(Color::Red)))
        }
        Some(GpuInfo::Loading) => Cell::from(Span::styled(
            "Loading...",
            Style::default().fg(Color::Yellow),
        )),
        None => Cell::from("Unknown"),
    };

    Row::new(vec![
        Cell::from(info.name.clone()),
        Cell::from(user_at_host),
        status_cell,
        cpu_cell,
        disk_cell,
        os_cell,
        gpu_cell,
    ])
    .style(Style::default().bg(bg))
    .height(2)
}
