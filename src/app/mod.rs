mod ssh_details;
mod ssh_list;
mod states;
use crate::app::states::{
    SharedCpuInfo, SharedDiskInfo, SharedGpuInfo, SharedOsInfo, SharedSshHosts, SharedSshStatuses,
    load_ssh_configs,
};
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
use tasks::disk_task::DiskInfoTask;
use tasks::executor::TaskExecutor;
use tasks::gpu_task::GpuInfoTask;
use tasks::os_task::OsInfoTask;
use tasks::ssh_status_task::SshStatusTask;

#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    List,
    Detail,
    Search,
}

pub struct App {
    running: bool,
    event_stream: EventStream,
    pub ssh_hosts: SharedSshHosts,
    pub ssh_statuses: SharedSshStatuses,
    pub cpu_info: SharedCpuInfo,
    pub disk_info: SharedDiskInfo,
    pub os_info: SharedOsInfo,
    pub gpu_info: SharedGpuInfo,
    pub selected_id: Option<String>,
    pub search_query: String,
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
            disk_info: Arc::new(Mutex::new(HashMap::new())),
            os_info: Arc::new(Mutex::new(HashMap::new())),
            gpu_info: Arc::new(Mutex::new(HashMap::new())),
            running: false,
            event_stream: EventStream::new(),
            selected_id,
            search_query: String::new(),
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
        executor.register(DiskInfoTask {
            ssh_hosts: Arc::clone(&self.ssh_hosts),
            disk_info: Arc::clone(&self.disk_info),
        });
        executor.register(OsInfoTask {
            ssh_hosts: Arc::clone(&self.ssh_hosts),
            os_info: Arc::clone(&self.os_info),
        });
        executor.register(GpuInfoTask {
            ssh_hosts: Arc::clone(&self.ssh_hosts),
            gpu_info: Arc::clone(&self.gpu_info),
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
            AppMode::List | AppMode::Search => render_list(self, frame),
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
            AppMode::List => match key.code {
                KeyCode::Char('/') => {
                    self.mode = AppMode::Search;
                    self.search_query.clear();
                }
                _ => handle_list_key(self, key),
            },
            AppMode::Search => match key.code {
                KeyCode::Esc => {
                    self.mode = AppMode::List;
                    self.search_query.clear();
                }
                KeyCode::Enter => {
                    self.mode = AppMode::List;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            },
            AppMode::Detail => {
                if key.code == KeyCode::Esc {
                    self.mode = AppMode::List;
                }
            }
        }
    }
}
