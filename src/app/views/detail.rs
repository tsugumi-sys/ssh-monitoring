use ratatui::{prelude::*, widgets::Block};
use crate::app::App;

pub fn draw(_app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let block = Block::bordered().title("SSH Host Detail View - Press Esc to return");
    frame.render_widget(block, area);
}
