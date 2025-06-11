use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use lyrics_next::client::get_lyrics_client;
use lyrics_next::config::Config;
use lyrics_next::log::init_logger;
use lyrics_next::ui::App;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    let args = Args::parse();
    Config::load_or_default(args.config)?;
    get_lyrics_client();
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore();
    app_result
}
