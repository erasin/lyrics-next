use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::Deserialize;

use super::{BaseFetcher, LyricsFetcher};
use crate::{error::LyricsError, song::SongInfo};

#[derive(Debug, Deserialize)]
struct SearchResponse {
    data: SearchData,
}

#[derive(Debug, Deserialize)]
struct SearchData {
    info: Vec<Song>,
}

// data
#[derive(Debug, Deserialize)]
struct Song {
    hash: String,
    album_id: String,
    album_name: String,
    singername: String,
    // songname: String,
    songname_original: String,
}

#[derive(Debug, Deserialize)]
struct LyricResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    accesskey: String,
    download_id: String,
    singer: String,
    song: String,
}

#[derive(Debug, Deserialize)]
struct LyricData {
    content: String,
}

// Kugou音乐实现
#[derive(Default)]
pub(super) struct KugouFetcher {
    base: BaseFetcher,
}

impl KugouFetcher {
    // 酷狗歌词解密函数
    fn decode_lyric(&self, encrypted: &str) -> Result<String, LyricsError> {
        let bytes = BASE64_STANDARD.decode(encrypted)?;
        let re = String::from_utf8(bytes).map_err(|_| LyricsError::LyricsDecodeError)?;
        if re.is_empty() {
            return Err(LyricsError::NoLyricsFound);
        }
        Ok(re)
    }
}

#[async_trait]
impl LyricsFetcher for KugouFetcher {
    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError> {
        log::debug!("kugou start ");

        // 1. 搜索歌曲
        let search_url = "http://mobilecdn.kugou.com/api/v3/search/song";
        let request = self.base.client.get(search_url).query(&[
            (
                "keyword",
                format!("{} {}", song.title, song.artist).as_str(),
            ),
            ("page", "1"),
            ("pagesize", "1"),
        ]);

        let data: SearchResponse = self.base.fetch_with_retry(request).await?;
        log::debug!("song json: {:?}", data);

        let search = data
            .data
            .info
            .into_iter()
            .filter(|s| {
                s.songname_original == song.title || s.songname_original.contains(&song.title)
            })
            .filter(|s| {
                if song.artist.is_empty() {
                    true
                } else {
                    let a = s.singername.to_lowercase();
                    let b = song.artist.to_lowercase();
                    a == b || a.contains(&b)
                }
            })
            .filter(|s| {
                if song.album.is_empty() {
                    true
                } else {
                    let a = s.album_name.to_lowercase();
                    let b = song.album.to_lowercase();
                    a == b || a.contains(&b)
                }
            })
            .next()
            .map(|s| s)
            .ok_or(LyricsError::NoLyricsFound)?;

        log::debug!("song hash: {} {}", search.album_id, search.hash);

        // 2. 获取歌词
        let lyric_url = "http://krcs.kugou.com/search";
        let request = self
            .base
            .client
            .get(lyric_url)
            .query(&[
                ("hash", search.hash.as_str()),
                ("album_id", search.album_id.as_str()),
                ("ver", "1"),
                ("client", "pc"),
                ("man", "yes"),
            ])
            .header("User-Agent", "Mozilla/5.0");

        let data: LyricResponse = self.base.fetch_with_retry(request).await?;
        log::debug!("lyric list: {:?}", data);

        let lyric = data
            .candidates
            .into_iter()
            .filter(|s| s.song == song.title)
            .filter(|s| {
                if song.artist.is_empty() {
                    true
                } else {
                    let a = s.singer.to_lowercase();
                    let b = song.artist.to_lowercase();
                    a == b || a.contains(&b)
                }
            })
            .next()
            .map(|s| s)
            .ok_or(LyricsError::NoLyricsFound)?;

        log::debug!("Down id: {} , {}", lyric.download_id, lyric.accesskey);

        // 3. 下载
        let lyric_download_url = "http://lyrics.kugou.com/download";
        let request = self
            .base
            .client
            .get(lyric_download_url)
            .query(&[
                ("ver", "1"),
                ("client", "pc"),
                ("fmt", "lrc"),
                ("charset", "utf8"),
                ("accesskey", lyric.accesskey.as_str()),
                ("id", lyric.download_id.as_str()),
            ])
            .header("User-Agent", "Mozilla/5.0");

        let data: LyricData = self.base.fetch_with_retry(request).await?;
        log::debug!("lyric: {:?}", data);

        let decoded = self.decode_lyric(&data.content)?;
        Ok(decoded)
    }

    fn source_name(&self) -> &'static str {
        "Kugou"
    }
}
