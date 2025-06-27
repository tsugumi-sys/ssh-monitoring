use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut Frame) {
    let area = frame.area();

    // Split screen: title and host cards
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Cards
        ])
        .split(area);

    // Title
    let title = Paragraph::new("SSH Host List")
        .style(Style::default().add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(title, chunks[0]);

    // Layout for each host card
    let card_height = 6;
    let constraints = vec![Constraint::Length(card_height); app.ssh_hosts.len()];
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(chunks[1]);

    for (i, (host_state, chunk)) in app.ssh_hosts.iter().zip(vertical_chunks.iter()).enumerate() {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(host_state.info.name.clone())
            .border_style(if i == app.selected_index {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default()
            });

        let content = Paragraph::new(format!(
            "User: {}\nIP: {}\nPort: {}\nStatus: {:?}",
            host_state.info.user, host_state.info.ip, host_state.info.port, host_state.status,
        ))
        .block(block)
        .wrap(Wrap { trim: true });

        frame.render_widget(content, *chunk);
    }
}
