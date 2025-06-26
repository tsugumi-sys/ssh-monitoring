// src/main.rs
use color_eyre::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use futures::{FutureExt, StreamExt};
use ratatui::{
    layout::Constraint,
    prelude::*,
    widgets::{Block, Row, Table},
};

mod ssh_config;
use ssh_config::load_ssh2_config_hosts;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

pub struct App {
    running: bool,
    event_stream: EventStream,
    ssh_hosts: Vec<(String, Option<String>, Option<u16>, Option<String>)>,
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
        let area = frame.area();
        let headers = ["Host", "IP Address", "Port", "Username"];

        let rows: Vec<Row> = self
            .ssh_hosts
            .iter()
            .map(|(host, ip, port, user)| {
                Row::new(vec![
                    host.clone(),
                    ip.clone().unwrap_or_else(|| "-".into()),
                    port.map(|p| p.to_string()).unwrap_or_else(|| "-".into()),
                    user.clone().unwrap_or_else(|| "-".into()),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Length(6),
                Constraint::Percentage(20),
            ],
        )
        .header(Row::new(headers).bold())
        .block(Block::bordered().title("SSH Hosts"));

        frame.render_widget(table, area);
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
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                self.running = false
            }
            _ => {}
        }
    }
}
