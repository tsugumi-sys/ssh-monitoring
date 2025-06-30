use crate::app::{App, AppMode};
use crossterm::event::KeyCode;

// Must match your layout!
const COLUMNS: usize = 3;

pub fn handle_key(app: &mut App, key: crossterm::event::KeyEvent) {
    let hosts_guard = futures::executor::block_on(app.ssh_hosts.lock());

    let mut host_entries: Vec<_> = hosts_guard.iter().collect(); // Vec<(&String, &SshHostState)>
    host_entries.sort_by_key(|(_, h)| &h.name);

    let total = host_entries.len();
    if total == 0 {
        return;
    }

    // Find current selected index
    let current_index = app
        .selected_id
        .as_ref()
        .and_then(|id| host_entries.iter().position(|(key, _)| *key == id))
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

    // Update selection
    if let Some((id, _)) = host_entries.get(next_index) {
        app.selected_id = Some(id.to_string());
    }
}
