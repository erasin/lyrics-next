use std::path::PathBuf;

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use lyrics_next::client::get_lyrics_client;
use lyrics_next::config::{Config, log_path};
use lyrics_next::ui::App;
use tracing::{Level, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::FormatTime;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
    // line
}

/// 日志时间
pub struct LocalTimer;

impl FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S"))
    }
}

pub fn init_logger() -> Result<()> {
    #[cfg(debug_assertions)]
    let level = Level::DEBUG;

    #[cfg(not(debug_assertions))]
    let level = Level::INFO;

    // 创建按日轮转的日志文件 (存储在./logs目录)
    let file_appender = RollingFileAppender::new(
        Rotation::NEVER,
        log_path(),
        format!("lyrics-{}.log", Local::now().format("%Y%m%d")),
    );

    tracing_subscriber::fmt()
        .with_timer(LocalTimer)
        .with_max_level(level)
        .with_target(false)
        .with_file(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_ansi(false)
        // .with_writer(std::io::stdout)
        .with_writer(file_appender)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    info!("Starting lyric application...");
    let args = Args::parse();
    Config::load_or_default(args.config)?;
    get_lyrics_client();
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore();
    app_result
}
