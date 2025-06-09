use std::path::PathBuf;

use sanitize_filename::sanitize;

use crate::{config::cache_path, error::LyricsError, song::SongInfo};

// 缓存管理模块
#[derive(Debug, Clone, Default)]
pub struct CacheManager {
    base_dir: PathBuf,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            base_dir: cache_path(),
        }
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

    pub async fn get(&self, song: &SongInfo) -> Option<String> {
        let path = self.lyrics_name(song);
        if !path.exists() {
            return None;
        }
        tokio::fs::read_to_string(&path).await.ok()
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

    pub async fn delete(&self, song: &SongInfo) {
        let path = self.lyrics_name(song);
        match tokio::fs::remove_file(path).await {
            Ok(_) => {}
            Err(e) => tracing::error!("delete file {} failed {}", song.title, e),
        }
    }
}
