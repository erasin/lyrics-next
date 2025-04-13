use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};

use anyhow::Result;
use chrono::Local;
use clap::Parser;
use lyrics_next::client::get_lyrics_client;
use lyrics_next::config::{Config, get_config, log_path};
use lyrics_next::ui::App;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
    // line
}

pub fn init_logger() -> Result<()> {
    // 日志文件路径（用户目录下的 .lyrics/logs/app.log）
    let log_file = log_path();

    #[cfg(debug_assertions)]
    let level = log::LevelFilter::Trace;

    #[cfg(not(debug_assertions))]
    let level = log::LevelFilter::Info;

    // 配置日志输出到文件和终端
    env_logger::Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or(""),
                record.args()
            )
        })
        .filter(None, level) // 默认日志级别
        .target(env_logger::Target::Pipe(Box::new(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)?,
        )))
        .try_init()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logger()?;
    log::info!("Starting lyric application...");
    get_lyrics_client();
    let args = Args::parse();

    if let Some(config_path) = args.config {
        Config::load(&config_path)?;
    } else {
        Config::load_default()?;
    }

    log::debug!("config: {:?}", get_config());

    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal).await;
    ratatui::restore();
    app_result
}
