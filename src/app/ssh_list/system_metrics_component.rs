use crate::app::states::{CpuInfo, SshHostInfo};
use ratatui::prelude::*;
use ratatui::text::{Line, Span};

pub fn render_system_metrics_lines<'a>(
    info: &'a SshHostInfo,
    cpu_info: Option<&'a CpuInfo>,
) -> Vec<Line<'a>> {
    let mut lines = vec![Line::from(Span::styled(
        "System Metrics",
        Style::default().add_modifier(Modifier::UNDERLINED),
    ))];

    if info.is_placeholder_identity_file() {
        lines.push(Line::from(Span::styled(
            "Not available (no identity file set)",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )));
        return lines;
    }

    match cpu_info {
        Some(CpuInfo::Loading) => {
            lines.push(Line::from("CPU: Loading..."));
        }
        Some(CpuInfo::Success {
            core_count,
            usage_percent,
        }) => {
            lines.push(Line::from(format!(
                "CPU: {} cores, {:.1}% usage",
                core_count, usage_percent
            )));
        }
        Some(CpuInfo::Failure(e)) => {
            lines.push(Line::from(Span::styled(
                format!("CPU: Failed - {}", e),
                Style::default().fg(Color::Red),
            )));
        }
        None => {
            lines.push(Line::from("CPU: Unknown"));
        }
    }

    lines
}
