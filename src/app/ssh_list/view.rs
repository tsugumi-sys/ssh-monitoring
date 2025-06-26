use crate::app::App;
use ratatui::Frame;
use ratatui::prelude::Constraint;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::widgets::*;

pub fn render(app: &App, frame: &mut Frame) {
    let rows: Vec<Row> = app
        .ssh_hosts
        .iter()
        .enumerate()
        .map(|(i, host)| {
            Row::new(vec![
                Cell::from(host.name.clone()),
                Cell::from(host.user.clone()),
                Cell::from(host.ip.clone()),
                Cell::from(host.port.to_string()),
            ])
            .style(if i == app.selected_index {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            })
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Length(6),
            Constraint::Percentage(20),
        ],
    )
    .header(Row::new(vec!["Name", "User", "IP", "Port"]))
    .block(Block::default().borders(Borders::ALL).title("SSH Hosts"))
    .highlight_symbol(">>");

    frame.render_widget(table, frame.area());
}
