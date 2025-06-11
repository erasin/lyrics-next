use anyhow::Result;
use chrono::Local;
use tracing::Level;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::fmt::time::FormatTime;

use crate::config::log_path;

/// 日志时间
struct LocalTimer;

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
