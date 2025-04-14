use std::sync::{RwLock, RwLockWriteGuard};

use thiserror::Error;

use crate::config::Config;

#[derive(Error, Debug)]
pub enum LyricsError {
    #[error("error: {0}")]
    AnyError(#[from] anyhow::Error),

    #[error("MPRIS error: {0}")]
    MprisError(#[from] mpris::DBusError),

    #[error("MPRIS Find error: {0}")]
    MprisFindError(#[from] mpris::FindingError),

    #[error("HTTP error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("base64 error: {0}")]
    DecodeError(#[from] base64::DecodeError),

    #[error("No active media player found")]
    NoPlayerFound,

    #[error("Failed to get cache path")]
    CachePathError,

    #[error("No lyrics found")]
    NoLyricsFound,

    #[error("JSON parse error")]
    JsonError,

    #[error("Lyrics validation failed")]
    LyricsValidationFailed,

    #[error("Lyrics decode failed")]
    LyricsDecodeError,

    #[error("Invalid time format")]
    InvalidTimeFormat,

    #[error("Empty lyrics content")]
    EmptyLyrics,
}
