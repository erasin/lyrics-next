use std::sync::OnceLock;

use async_trait::async_trait;
use kugou::KugouFetcher;
use netease::NeteaseFetcher;
use qqmusic::QQMusicFetcher;
use reqwest::RequestBuilder;
use serde::de::DeserializeOwned;

use crate::{
    cache::CacheManager, config::get_config, error::LyricsError, song::SongInfo,
    utils::normalize_text,
};

mod kugou;
mod netease;
mod ovh;
mod qqmusic;

/// 歌词抓取器
#[async_trait]
trait LyricsFetcher: Send + Sync {
    async fn search_lyric(&self, song: &SongInfo) -> Result<Vec<LyricsItem>, LyricsError>;
    async fn download_lyric(&self, item: &LyricsItem) -> Result<String, LyricsError>;
    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError>;
    fn source_name(&self) -> &'static str;
}

#[derive(Debug, Clone)]
pub struct LyricsItem {
    pub source: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub params: Vec<(String, String)>,
}

// 公共基础结构
struct BaseFetcher {
    client: reqwest::Client,
    retries: u8,
}

impl Default for BaseFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseFetcher {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            retries: 3,
        }
    }

    // 添加重试机制
    async fn fetch_with_retry<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T, LyricsError> {
        let mut attempt = 0;
        loop {
            let response = request.try_clone().unwrap().send().await;
            log::debug!("REQUEST: {:?} \n RESPONSE: {:?}", request, response);
            match response {
                Ok(res) => return Ok(res.json::<T>().await?),
                Err(_e) if attempt < self.retries => {
                    tokio::time::sleep(std::time::Duration::from_secs(1 << attempt)).await;
                    attempt += 1;
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}

/// 初始client
pub fn get_lyrics_client() -> &'static LyricsClient {
    static CLIENT: OnceLock<LyricsClient> = OnceLock::new();
    CLIENT.get_or_init(LyricsClient::new)
}

// 统一调用入口
pub struct LyricsClient {
    fetchers: Vec<Box<dyn LyricsFetcher>>,
    pub cache: CacheManager,
}

impl LyricsClient {
    fn new() -> Self {
        let mut fetchers: Vec<Box<dyn LyricsFetcher>> = Vec::new();

        let config = &get_config().read().unwrap().sources;

        if config.kugou {
            fetchers.push(Box::new(KugouFetcher::default()));
        }

        if config.netease {
            fetchers.push(Box::new(NeteaseFetcher::default()));
        }

        if config.qq {
            fetchers.push(Box::new(QQMusicFetcher::default()));
        }

        Self {
            fetchers,
            cache: CacheManager::new(),
        }
    }

    pub async fn get_search(&self, song: &SongInfo) -> Result<Vec<LyricsItem>, LyricsError> {
        let mut list = Vec::new();

        for fetcher in &self.fetchers {
            let mut sl = fetcher.search_lyric(song).await.unwrap_or_default();
            list.append(&mut sl);
        }

        Ok(list)
    }

    pub async fn get_lyrics(&self, song: &SongInfo) -> Result<String, LyricsError> {
        if let Some(cached) = self.cache.get(song).await {
            log::debug!("Cache lyric for: {} - {}", song.artist, song.title);
            return Ok(cached);
        }

        for fetcher in &self.fetchers {
            log::info!("Trying source: {}", fetcher.source_name());
            match fetcher.fetch_lyric(song).await {
                Ok(lyric) => {
                    //if self.validate_lyric(song, &lyric) {
                    log::info!("Successfully fetched from {}", fetcher.source_name());
                    self.cache
                        .store(song, fetcher.source_name(), &lyric)
                        .await?;
                    return Ok(lyric);
                    // }
                }
                Err(e) => log::warn!("{} failed: {}", fetcher.source_name(), e),
            }
        }
        Err(LyricsError::NoLyricsFound)
    }

    pub async fn download(&self, song: &SongInfo, item: &LyricsItem) -> Result<(), LyricsError> {
        for fetcher in &self.fetchers {
            if fetcher.source_name() == item.source {
                match fetcher.download_lyric(item).await {
                    Ok(lyric) => {
                        log::info!("Successfully fetched from {}", fetcher.source_name());
                        self.cache
                            .store(song, fetcher.source_name(), &lyric)
                            .await?;
                        return Ok(());
                    }
                    Err(e) => log::warn!("{} failed: {}", fetcher.source_name(), e),
                }
            }
        }

        Err(LyricsError::NoLyricsFound)
    }

    #[allow(dead_code)]
    fn validate_lyric(&self, song: &SongInfo, lyric: &str) -> bool {
        let normalized_lyric = normalize_text(lyric);
        let has_title = normalized_lyric.contains(&normalize_text(&song.title));
        let has_artist = normalized_lyric.contains(&normalize_text(&song.artist));

        // 额外检查时长标签（如果有）
        let has_duration = lyric.contains(&format!("{:0.1}", song.duration));

        has_title && has_artist && (song.duration <= 0.0 || has_duration)
    }
}

// 检测两者是否类似

pub fn get_first(list: Vec<LyricsItem>, song: &SongInfo) -> Result<LyricsItem, LyricsError> {
    let list: Vec<LyricsItem> = list
        .into_iter()
        .filter(|s| s.title == song.title || s.title.contains(&song.title))
        .collect();

    if list.is_empty() {
        return Err(LyricsError::NoLyricsFound);
    }

    let list: Vec<LyricsItem> = if !song.artist.is_empty() {
        list.into_iter()
            .filter(|s| {
                if song.artist.is_empty() {
                    true
                } else {
                    let a = s.artist.to_lowercase();
                    let b = song.artist.to_lowercase();
                    a == b || a.contains(&b)
                }
            })
            .collect()
    } else {
        list
    };

    if list.is_empty() {
        return Err(LyricsError::NoLyricsFound);
    }

    let list: Vec<LyricsItem> = if !song.album.is_empty() {
        list.into_iter()
            .filter(|s| {
                if song.album.is_empty() {
                    true
                } else {
                    let a = s.album.to_lowercase();
                    let b = song.album.to_lowercase();
                    a == b || a.contains(&b)
                }
            })
            .collect()
    } else {
        list
    };

    if list.is_empty() {
        return Err(LyricsError::NoLyricsFound);
    }

    let first = list.first().ok_or(LyricsError::NoLyricsFound)?;
    Ok(first.clone())
}
