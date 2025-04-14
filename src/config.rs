use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
    fs,
    path::PathBuf,
    sync::{OnceLock, RwLock},
};

use crate::{error::LyricsError, utils::ensure_parent_dir};

/// config
static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();

pub fn get_config() -> &'static RwLock<Config> {
    CONFIG.get_or_init(|| RwLock::new(Config::default()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct Config {
    pub ui: Ui,
    pub sources: Sources,
}

#[derive(Debug, Deserialize)]
pub struct Ui {
    pub title: bool,
    pub time: bool,
    pub progress_bar: bool,
}

#[derive(Debug, Deserialize)]
pub struct Sources {
    pub netease: bool,
    pub qq: bool,
    pub kugou: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            ui: Ui {
                title: true,
                time: false,
                progress_bar: true,
            },
            sources: Sources {
                netease: true,
                qq: true,
                kugou: true,
            },
        }
    }
}

impl Config {
    pub fn load_or_default(path: Option<PathBuf>) -> Result<(), LyricsError> {
        let config_path = match path {
            Some(p) => p,
            None => config_path(),
        };

        let config: Config = if config_path.exists() {
            let config_content = fs::read_to_string(&config_path).with_context(|| {
                format!("Failed to read config file: {}", config_path.display())
            })?;

            toml::from_str(&config_content).with_context(|| {
                format!("Failed to parse config file: {}", config_path.display())
            })?
        } else {
            Self::default()
        };

        let mut c = get_config().write().expect("Get config failed.");
        *c = config;

        Ok(())
    }
}

pub fn config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("lyrics/lyrics.toml");
    ensure_parent_dir(&config_dir);
    config_dir
}

pub fn log_path() -> PathBuf {
    let log_file = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".lyrics/lyrics.log");
    ensure_parent_dir(&log_file);
    log_file
}

pub fn cache_path() -> PathBuf {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".lyrics");
    ensure_parent_dir(&cache_dir);
    cache_dir
}
