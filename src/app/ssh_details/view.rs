use crate::app::App;
use ratatui::Frame;
use ratatui::widgets::{Block, Borders};

pub fn render(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let hosts = futures::executor::block_on(app.ssh_hosts.lock());

    let selected_name = app
        .selected_id
        .as_ref()
        .and_then(|id| hosts.get(id))
        .map(|h| h.info.name.clone())
        .unwrap_or_else(|| "<none>".to_string());

    let block = Block::default()
        .title(format!(
            "SSH Host Detail View - Press Esc to return (Selected: {})",
            selected_name
        ))
        .borders(Borders::ALL);

    frame.render_widget(block, area);
}
