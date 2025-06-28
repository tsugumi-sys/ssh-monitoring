mod ssh_details;
mod ssh_list;
mod states;
use crate::app::states::{SharedCpuInfo, SharedSshHosts, SharedSshStatuses, load_ssh_configs};
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind};
use futures::{FutureExt, StreamExt};
use ratatui::prelude::*;
use ssh_details::render as render_detail;
use ssh_list::{handle_key as handle_list_key, render as render_list};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
mod tasks;
use tasks::cpu_status_task::CpuInfoTask;
use tasks::executor::TaskExecutor;
use tasks::ssh_status_task::SshStatusTask;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    List,
    Detail,
}

pub struct App {
    running: bool,
    event_stream: EventStream,
    pub ssh_hosts: SharedSshHosts,
    pub ssh_statuses: SharedSshStatuses,
    pub cpu_info: SharedCpuInfo,
    pub selected_id: Option<String>,
    pub scroll_offset: usize,
    pub visible_rows: usize,
    pub mode: AppMode,
}

impl App {
    pub fn new() -> Self {
        let ssh_hosts = load_ssh_configs().unwrap_or_default(); // now a HashMap
        let selected_id = ssh_hosts.keys().next().cloned();

        Self {
            ssh_hosts: Arc::new(Mutex::new(ssh_hosts)),
            ssh_statuses: Arc::new(Mutex::new(HashMap::new())),
            cpu_info: Arc::new(Mutex::new(HashMap::new())),
            running: false,
            event_stream: EventStream::new(),
            scroll_offset: 0,
            visible_rows: 0,
            selected_id,
            mode: AppMode::List,
        }
    }

    pub async fn run(
        mut self,
        mut terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        self.running = true;

        let mut executor = TaskExecutor::new();
        executor.register(SshStatusTask {
            ssh_hosts: Arc::clone(&self.ssh_hosts),
            ssh_statuses: Arc::clone(&self.ssh_statuses),
        });
        executor.register(CpuInfoTask {
            ssh_hosts: Arc::clone(&self.ssh_hosts),
            cpu_info: Arc::clone(&self.cpu_info),
        });
        executor.start();

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
