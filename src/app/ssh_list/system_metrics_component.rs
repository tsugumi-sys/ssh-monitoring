use crate::app::states::{CpuInfo, DiskInfo, GpuInfo, OsInfo, SshHostInfo};
use ratatui::prelude::*;
use ratatui::text::{Line, Span};

pub fn render_system_metrics_lines<'a>(
    info: &'a SshHostInfo,
    cpu_info: Option<&'a CpuInfo>,
    disk_info: Option<&'a DiskInfo>,
    os_info: Option<&'a OsInfo>,
    gpu_info: Option<&'a GpuInfo>,
) -> Vec<Line<'a>> {
    if info.is_placeholder_identity_file() {
        return vec![Line::from(Span::styled(
            "Metrics unavailable",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    let mut lines = Vec::new();

    let format_line = |label: &str, value: String, style: Option<Style>| {
        let padded_label = format!("{:<5}", format!("{}:", label)); // label + ':' padded to 5 chars
        let spacer = "     "; // 5 spaces between label and value
        let full_text = format!("{}{}{}", padded_label, spacer, value);

        match style {
            Some(style) => Line::from(Span::styled(full_text, style)),
            None => Line::from(full_text),
        }
    };

    // OS
    lines.push(match os_info {
        Some(OsInfo::Success { name, version, .. }) => {
            format_line("OS", format!("{} {}", name, version), None)
        }
        Some(OsInfo::Failure(e)) => format_line(
            "OS",
            format!("Failed ({})", e),
            Some(Style::default().fg(Color::Red)),
        ),
        Some(OsInfo::Loading) => format_line("OS", "Loading...".into(), None),
        None => format_line("OS", "Unknown".into(), None),
    });

    // CPU
    lines.push(match cpu_info {
        Some(CpuInfo::Success {
            core_count,
            usage_percent,
        }) => format_line("CPU", format!("{core_count}c, {usage_percent:.0}%"), None),
        Some(CpuInfo::Failure(e)) => format_line(
            "CPU",
            format!("Failed ({})", e),
            Some(Style::default().fg(Color::Red)),
        ),
        Some(CpuInfo::Loading) => format_line("CPU", "Loading...".into(), None),
        None => format_line("CPU", "Unknown".into(), None),
    });

    // Disk
    lines.push(match disk_info {
        Some(DiskInfo::Success { used, total, .. }) => {
            format_line("Disk", format!("{used}/{total}"), None)
        }
        Some(DiskInfo::Failure(e)) => format_line(
            "Disk",
            format!("Failed ({})", e),
            Some(Style::default().fg(Color::Red)),
        ),
        Some(DiskInfo::Loading) => format_line("Disk", "Loading...".into(), None),
        None => format_line("Disk", "Unknown".into(), None),
    });

    // GPU
    lines.push(match gpu_info {
        Some(GpuInfo::Success {
            utilization_percent,
            temperature_c,
            ..
        }) => format_line(
            "GPU",
            format!("{utilization_percent}% @ {temperature_c}°C"),
            None,
        ),
        Some(GpuInfo::Failure(e)) => format_line(
            "GPU",
            format!("Failed ({})", e),
            Some(Style::default().fg(Color::Red)),
        ),
        Some(GpuInfo::Fallback(text)) => format_line("GPU", text.clone(), None),
        Some(GpuInfo::Loading) => format_line("GPU", "Loading...".into(), None),
        None => format_line("GPU", "Unknown".into(), None),
    });

    lines
}
