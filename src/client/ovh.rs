use async_trait::async_trait;
use serde_json::Value;

use super::{BaseFetcher, LyricsFetcher, LyricsItem};
use crate::{error::LyricsError, song::SongInfo};

// Spotify音乐实现
#[allow(dead_code)]
#[derive(Default)]
pub(super) struct OvhFetcher {
    base: BaseFetcher,
}

#[async_trait]
impl LyricsFetcher for OvhFetcher {
    async fn search_lyric(&self, song: &SongInfo) -> Result<Vec<LyricsItem>, LyricsError> {
        Err(LyricsError::NoLyricsFound)
    }
    async fn download_lyric(&self, item: &LyricsItem) -> Result<String, LyricsError> {
        Err(LyricsError::NoLyricsFound)
    }
    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError> {
        // 假设使用的第三方Spotify歌词API如下（实际应使用真实的API）
        let ovh_api = "https://api.lyrics.ovh/v1";

        let api_url = format!("{}/{}/{}", ovh_api, song.artist, song.title);

        // let encoded_artist = urlencoding::encode(&song.artist);
        // let encoded_title = urlencoding::encode(&song.title);
        let request = self
            .base
            .client
            .get(api_url)
            // .query(&[("track", &song.title), ("artist", &song.artist)])
            .header("Accept", "application/json")
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            );

        let json: Value = self.base.fetch_with_retry(request).await?;
        let lyrics = json["lyrics"].as_str().ok_or(LyricsError::NoLyricsFound)?;

        if lyrics.is_empty() {
            return Err(LyricsError::NoLyricsFound);
        }

        // 假设第三方API返回的歌词不需要解码或特殊处理
        Ok(lyrics.to_string())
    }

    fn source_name(&self) -> &'static str {
        "Spotify"
    }
}
