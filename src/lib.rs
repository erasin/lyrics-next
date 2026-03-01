pub mod cache;
pub mod client;
pub mod config;
pub mod error;
pub mod log;
pub mod player;
pub mod song;
pub mod ui;
pub(crate) mod utils;

rust_i18n::i18n!("locales", fallback = "zh");
