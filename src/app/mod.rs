pub mod views;

use crate::ssh_config::load_ssh2_config_hosts;
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{prelude::*, widgets::TableState};
use views::{draw_detail, draw_list};

#[derive(PartialEq)]
pub enum AppMode {
    List,
    Detail,
}

pub struct App {
    running: bool,
    event_stream: EventStream,
    pub ssh_hosts: Vec<(String, Option<String>, Option<u16>, Option<String>)>,
    pub selected_index: usize,
    pub mode: AppMode,
}

impl App {
    pub fn new() -> Self {
        let ssh_hosts = load_ssh2_config_hosts().unwrap_or_else(|err| {
            eprintln!("Failed to load SSH config: {err}");
            vec![]
        });
        Self {
            ssh_hosts,
            running: false,
            event_stream: EventStream::new(),
            selected_index: 0,
            mode: AppMode::List,
        }
    }

    pub async fn run(
        mut self,
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_crossterm_events().await?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        match self.mode {
            AppMode::List => draw_list(self, frame),
            AppMode::Detail => draw_detail(self, frame),
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
            AppMode::List => match key.code {
                KeyCode::Down => {
                    if self.selected_index + 1 < self.ssh_hosts.len() {
                        self.selected_index += 1;
                    }
                }
                KeyCode::Up => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                KeyCode::Enter => {
                    self.mode = AppMode::Detail;
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.running = false;
                }
                _ => {}
            },
            AppMode::Detail => match key.code {
                KeyCode::Esc => {
                    self.mode = AppMode::List;
                }
                _ => {}
            },
        }
    }
}
