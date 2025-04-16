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

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "kebab-case", default, deny_unknown_fields)]
pub struct Config {
    pub player_filter: PlayerFilter,
    pub ui: Ui,
    pub sources: Sources,
}

#[derive(Debug, Deserialize)]
pub struct PlayerFilter {
    #[serde(default = "Vec::new")]
    pub only: Vec<String>,
    #[serde(default = "default_player_except")]
    pub except: Vec<String>,
}

fn default_player_except() -> Vec<String> {
    vec![
        "browser".to_string(),
        "video".to_string(),
        "screen-cast".to_string(),
        "chromium".to_string(),
        "firefox".to_string(),
    ]
}

impl Default for PlayerFilter {
    fn default() -> Self {
        PlayerFilter {
            only: vec![],
            except: default_player_except(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Ui {
    #[serde(default = "default_true")]
    pub title: bool,
    #[serde(default)]
    pub time: bool,
    #[serde(default = "default_true")]
    pub progress_bar: bool,
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            title: true,
            time: false,
            progress_bar: true,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Sources {
    #[serde(default = "default_true")]
    pub netease: bool,
    #[serde(default = "default_true")]
    pub qq: bool,
    #[serde(default = "default_true")]
    pub kugou: bool,
}

impl Default for Sources {
    fn default() -> Self {
        Sources {
            netease: true,
            qq: true,
            kugou: true,
        }
    }
}

fn default_true() -> bool {
    true
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

        log::debug!("config: {:?}", config);

        let mut c = get_config().write().expect("Get config failed.");
        *c = config;

        Ok(())
    }
}

const CONFIG_PATH: &str = ".lyrics";

pub fn config_path() -> PathBuf {
    let config_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_PATH)
        .join("lyrics.toml");
    ensure_parent_dir(&config_dir);
    config_dir
}

pub fn log_path() -> PathBuf {
    let log_file = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_PATH)
        .join("lyrics.log");
    ensure_parent_dir(&log_file);
    log_file
}

pub fn cache_path() -> PathBuf {
    let cache_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(CONFIG_PATH);
    ensure_parent_dir(&cache_dir);
    cache_dir
}
