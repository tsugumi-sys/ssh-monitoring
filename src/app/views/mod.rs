pub mod detail;
pub mod list;

use crate::app::App;
use ratatui::prelude::*;

pub fn draw_list(app: &mut App, frame: &mut Frame) {
    list::draw(app, frame);
}

pub fn draw_detail(app: &mut App, frame: &mut Frame) {
    detail::draw(app, frame);
}
