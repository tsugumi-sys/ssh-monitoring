use crate::app::states::SshHostInfo;
use ratatui::prelude::*;
use ratatui::text::{Line, Span};

pub fn render_host_info(info: &SshHostInfo) -> Vec<Line<'_>> {
    let mut lines = vec![
        Line::from(Span::styled(
            "Host Info",
            Style::default().add_modifier(Modifier::UNDERLINED),
        )),
        Line::from(format!("User: {}", info.user)),
        Line::from(format!("IP:   {}", info.ip)),
        Line::from(format!("Port: {}", info.port)),
        Line::from(format!("Key:  {}", info.identity_file)),
    ];

    if info.is_placeholder_identity_file() {
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
