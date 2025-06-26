use crate::app::App;
use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();

    let block = Block::default()
        .title(format!(
            "SSH Host Detail View - Press Esc to return (Selected: {})",
            app.ssh_hosts
                .get(app.selected_index)
                .map(|h| &h.name)
                .unwrap_or(&"<none>".to_string())
        ))
        .borders(Borders::ALL);

    frame.render_widget(block, area);
}
