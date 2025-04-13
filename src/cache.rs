use std::path::PathBuf;

use dirs::home_dir;
use ropey::Rope;
use sanitize_filename::sanitize;

use crate::{error::LyricsError, song::SongInfo};

/// 缓存文件
pub const CACHE_DIR: &str = ".lyrics";

// 缓存管理模块
#[derive(Debug, Clone, Default)]
pub struct CacheManager {
    base_dir: PathBuf,
}

impl CacheManager {
    pub fn new() -> Self {
        let mut path = home_dir().expect("Failed to get home directory");
        path.push(CACHE_DIR);

        if !path.exists() {
            std::fs::create_dir_all(&path).unwrap();
        }

        Self { base_dir: path }
    }

    fn lyrics_name(&self, song: &SongInfo) -> PathBuf {
        let mut name = vec![sanitize(&song.artist), sanitize(&song.title)];
        if !song.album.is_empty() {
            name.push(sanitize(&song.album));
        }
        let file_name = format!("{}.lrc", name.join("-"));
        let mut path = self.base_dir.clone();
        path.push(file_name);
        path
    }

    pub async fn get(&self, song: &SongInfo) -> Option<Rope> {
        let path = self.lyrics_name(song);
        if !path.exists() {
            return None;
        }

        tokio::fs::read_to_string(&path)
            .await
            .map(|s| Rope::from_str(&s))
            .ok()
    }

    pub async fn store(
        &self,
        song: &SongInfo,
        _source: &str,
        content: &str,
    ) -> Result<(), LyricsError> {
        let path = self.lyrics_name(song);
        tokio::fs::write(path, &content).await?;
        Ok(())
    }

    pub fn delete(&self, song: &SongInfo) {
        let path = self.lyrics_name(song);
        // tokio::fs::remove_file(path).await?
        match std::fs::remove_file(path) {
            Ok(_) => {}
            Err(e) => log::error!("delete file {} failed {}", song.title, e),
        }
    }
}
