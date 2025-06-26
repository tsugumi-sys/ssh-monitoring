use color_eyre::Result;
use ratatui::prelude::*;
use app::App;

mod ssh_config;
mod app;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal).await;
    ratatui::restore();
    result
}

