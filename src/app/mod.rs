mod ssh_details;
mod ssh_list;
mod states;
use crate::app::states::{SshHostState, load_ssh_host_states, update_ssh_status};
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::prelude::*;
use ssh_details::render as render_detail;
use ssh_list::{handle_key as handle_list_key, render as render_list};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Current screen the app is showing
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    List,
    Detail,
}

pub struct App {
    running: bool,
    event_stream: EventStream,
    pub ssh_hosts: Arc<Mutex<Vec<SshHostState>>>,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub visible_rows: usize,
    pub mode: AppMode,
}

impl App {
    pub fn new() -> Self {
        let ssh_hosts = load_ssh_host_states();
        Self {
            ssh_hosts: Arc::new(Mutex::new(ssh_hosts)),
            running: false,
            event_stream: EventStream::new(),
            selected_index: 0,
            scroll_offset: 0,
            visible_rows: 0,
            mode: AppMode::List,
        }
    }

    pub async fn run(
        mut self,
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.running = true;
        update_ssh_status(Arc::clone(&self.ssh_hosts));
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.mode {
            AppMode::List => render_list(self, frame),
            AppMode::Detail => render_detail(self, frame),
        }
    }

    async fn handle_crossterm_events(&mut self) -> Result<()> {
        tokio::select! {
            event = self.event_stream.next().fuse() => {
                if let Some(Ok(Event::Key(key))) = event {
                    if key.kind == KeyEventKind::Press {
                        self.on_key_event(key);
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_millis(100)) => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match self.mode {
            AppMode::List => handle_list_key(self, key),
            AppMode::Detail => {
                if key.code == KeyCode::Esc {
                    self.mode = AppMode::List;
                }
            }
        }
    }
}
