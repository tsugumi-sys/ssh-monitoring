use ratatui::{layout::Constraint, prelude::*, style::Style, widgets::{Block, Row, Table, TableState}};
use crate::app::App;

pub fn draw(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let headers = ["Host", "IP Address", "Port", "Username"];

    let rows: Vec<Row> = app
        .ssh_hosts
        .iter()
        .map(|(host, ip, port, user)| {
            Row::new(vec![
                host.clone(),
                ip.clone().unwrap_or_else(|| "-".into()),
                port.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                user.clone().unwrap_or_else(|| "-".into()),
            ])
        })
        .collect();

    let mut table_state = TableState::default();
    table_state.select(Some(app.selected_index));

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Length(6),
            Constraint::Percentage(20),
        ],
    )
    .header(Row::new(headers).bold())
    .highlight_style(Style::new().reversed())
    .highlight_symbol(">> ")
    .block(Block::bordered().title("SSH Hosts"));

    frame.render_stateful_widget(table, area, &mut table_state);
}
