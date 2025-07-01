use crate::app::states::SshHostInfo;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};

pub fn render_host_info(info: &SshHostInfo) -> Vec<Line<'_>> {
    let mut lines = vec![
        Line::from(format!("{}@{}:{}", info.user, info.ip, info.port)),
        Line::raw(""),
    ];

    if info.is_placeholder_identity_file() {
        lines.push(Line::from(Span::styled(
            "⚠ No IdentityFile",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    lines
}
