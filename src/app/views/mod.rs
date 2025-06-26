pub mod list;
pub mod detail;

use crate::app::{App, AppMode};
use ratatui::prelude::*;

pub fn draw_list(app: &mut App, frame: &mut Frame) {
    list::draw(app, frame);
}

pub fn draw_detail(app: &mut App, frame: &mut Frame) {
    detail::draw(app, frame);
}
