use crate::app::App;
use ratatui::{
    layout::Constraint,
    prelude::*,
    style::Style,
    widgets::{Block, Row, Table, TableState},
};

pub fn draw(app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let headers = ["Host", "IP Address", "Port", "Username"];

    let rows: Vec<Row> = app
        .ssh_hosts
        .iter()
        .map(|host| {
            Row::new(vec![
                host.name.clone(),
                host.ip.clone(),
                host.port.to_string(),
                host.user.clone(),
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
    .row_highlight_style(Style::new().reversed())
    .highlight_symbol(">> ")
    .block(Block::bordered().title("SSH Hosts"));

    frame.render_stateful_widget(table, area, &mut table_state);
}
