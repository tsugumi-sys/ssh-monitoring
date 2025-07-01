use crate::app::{App, AppMode};
use crossterm::event::KeyCode;

// Must match your layout!
const COLUMNS: usize = 3;

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let total = app.visible_hosts.len();
    if total == 0 {
        return;
    }

    let current_index = app
        .selected_id
        .as_ref()
        .and_then(|id| app.visible_hosts.iter().position(|(key, _)| key == id))
        .unwrap_or(0);

    let mut next_index = current_index;

    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            next_index = current_index + COLUMNS;
        }
        KeyCode::Char('k') | KeyCode::Up => {
            next_index = current_index.saturating_sub(COLUMNS);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            if (current_index + 1) % COLUMNS != 0 && current_index + 1 < total {
                next_index = current_index + 1;
            }
        }
        KeyCode::Char('h') | KeyCode::Left => {
            if current_index % COLUMNS != 0 {
                next_index = current_index - 1;
            }
        }
        KeyCode::Enter => {
            app.mode = AppMode::Detail;
            return;
        }
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
            return;
        }
        _ => return,
    }

    if let Some((id, _)) = app.visible_hosts.get(next_index) {
        app.selected_id = Some(id.clone());

        let selected_row = next_index / COLUMNS;
        let visible_rows = 5; // or compute dynamically

        if selected_row < app.vertical_scroll {
            app.vertical_scroll = selected_row;
        } else if selected_row >= app.vertical_scroll + visible_rows {
            app.vertical_scroll = selected_row + 1 - visible_rows;
        }
    }
}
