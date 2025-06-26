use crate::app::App;
use ratatui::{prelude::*, widgets::Block};

pub fn draw(_app: &mut App, frame: &mut Frame) {
    let area = frame.area();
    let block = Block::bordered().title("SSH Host Detail View - Press Esc to return");
    frame.render_widget(block, area);
}
