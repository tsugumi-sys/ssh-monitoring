use crate::app::{App, AppMode};
use crossterm::event::KeyCode;

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Down => {
            if app.selected_index + 1 < app.ssh_hosts.len() {
                app.selected_index += 1;
            }
        }
        KeyCode::Up => {
            if app.selected_index > 0 {
                app.selected_index -= 1;
            }
        }
        KeyCode::Enter => {
            app.mode = AppMode::Detail;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
        }
        _ => {}
    }
}
