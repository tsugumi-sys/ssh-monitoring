use crate::app::{App, AppMode};
use crossterm::event::KeyCode;

// Must match your layout!
const COLUMNS: usize = 3;

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let hosts = futures::executor::block_on(app.ssh_hosts.lock());
    let total = hosts.len();
    match key.code {
        // Move selection down (next row)
        KeyCode::Char('j') | KeyCode::Down => {
            let next = app.selected_index + COLUMNS;
            if next < total {
                app.selected_index = next;
                let selected_row = app.selected_index / COLUMNS;
                if selected_row >= app.scroll_offset + app.visible_rows {
                    app.scroll_offset += 1;
                }
            }
        }

        // Move selection up (previous row)
        KeyCode::Char('k') | KeyCode::Up => {
            if app.selected_index >= COLUMNS {
                app.selected_index -= COLUMNS;
                let selected_row = app.selected_index / COLUMNS;
                if selected_row < app.scroll_offset {
                    app.scroll_offset = app.scroll_offset.saturating_sub(1);
                }
            }
        }

        // Move right within row
        KeyCode::Char('l') | KeyCode::Right => {
            if (app.selected_index + 1) % COLUMNS != 0 && app.selected_index + 1 < total {
                app.selected_index += 1;
            }
        }

        // Move left within row
        KeyCode::Char('h') | KeyCode::Left => {
            if app.selected_index % COLUMNS != 0 {
                app.selected_index -= 1;
            }
        }

        // Enter detail mode
        KeyCode::Enter => {
            app.mode = AppMode::Detail;
        }

        // Quit
        KeyCode::Char('q') | KeyCode::Esc => {
            app.running = false;
        }

        KeyCode::PageDown => {
            let total = hosts.len();
            let rows = total.div_ceil(COLUMNS);
            let next_row = ((app.selected_index / COLUMNS) + app.visible_rows).min(rows - 1);
            app.selected_index = (next_row * COLUMNS).min(total - 1);
            app.scroll_offset = app.scroll_offset.saturating_add(app.visible_rows);
            if app.scroll_offset + app.visible_rows > rows {
                app.scroll_offset = rows.saturating_sub(app.visible_rows);
            }
        }

        KeyCode::PageUp => {
            if app.selected_index >= app.visible_rows * COLUMNS {
                let prev_row = (app.selected_index / COLUMNS).saturating_sub(app.visible_rows);
                app.selected_index = prev_row * COLUMNS;
                app.scroll_offset = app.scroll_offset.saturating_sub(app.visible_rows);
            } else {
                app.selected_index = 0;
                app.scroll_offset = 0;
            }
        }

        _ => {}
    }
}
