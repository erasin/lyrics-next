use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::Deserialize;

use super::{BaseFetcher, LyricsFetcher, LyricsItem};
use crate::{error::LyricsError, song::SongInfo};

#[derive(Debug, Deserialize)]
struct Response {
    data: SongResult,
}

// data
#[derive(Debug, Deserialize)]
struct SongResult {
    song: SongPack,
}

#[derive(Debug, Deserialize)]
struct SongPack {
    list: Vec<Song>,
}
#[derive(Debug, Deserialize)]
struct Song {
    songmid: String,
    songname: String,
    singer: Vec<Artist>,
    albumname: String,
}

#[derive(Debug, Deserialize)]
struct Artist {
    // id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct LyricsData {
    lyric: String,
}

// QQ音乐实现
#[derive(Default)]
pub(super) struct QQMusicFetcher {
    base: BaseFetcher,
}

#[async_trait]
impl LyricsFetcher for QQMusicFetcher {
    async fn search_lyric(&self, song: &SongInfo) -> Result<Vec<LyricsItem>, LyricsError> {
        Err(LyricsError::NoLyricsFound)
    }
    async fn download_lyric(&self, item: &LyricsItem) -> Result<String, LyricsError> {
        Err(LyricsError::NoLyricsFound)
    }
    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError> {
        log::debug!("QQ search");

        // 1. 搜索歌曲
        let search_url = "https://c.y.qq.com/soso/fcgi-bin/client_search_cp";
        let request= self
            .base
            .client
            .get(search_url)
            .query(&[
                ("w",  format!("{} {}", song.title, song.artist).as_str()),
                ("format", "json"),
                ("p","1"), // page
                ("n", "1"),// 每页数量
                ("cr", "1"), // 中文
                ("t","0") // 搜索类型 0 歌曲
                // ("g_tk", "5381"), //
            ])
            .header("Referer", "https://y.qq.com/n/ryqq/player")
            .header("Host", "c.y.qq.com")
            .header("Origin", "https://y.qq.com")
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/102.0.5005.63 Safari/537.36");

        let data = self.base.fetch_with_retry::<Response>(request).await?;
        log::debug!("Get song: {:?}, info: {:?}", data, song);

        let song_mid = data
            .data
            .song
            .list
            .into_iter()
            .filter(|s| s.songname == song.title || s.songname.contains(&song.title))
            .filter(|s| {
                if song.artist.is_empty() {
                    true
                } else {
                    s.singer.iter().find(|&a| a.name == song.artist).is_some()
                }
            })
            .filter(|s| {
                if song.album.is_empty() {
                    true
                } else {
                    let a = s.albumname.to_lowercase();
                    let b = song.album.to_lowercase();
                    a == b || a.contains(&b)
                }
            })
            .next()
            .map(|s| s.songmid)
            .ok_or(LyricsError::NoLyricsFound)?;

        log::debug!("song mid : {song_mid}");

        // 2. 获取歌词
        let lyrics_url = "https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg";
        let request = self
            .base
            .client
            .get(lyrics_url)
            .query(&[
                ("songmid", song_mid.as_str()),
                ("format", "json"),
                ("g_tk", "5381"),
            ])
            .header("Referer", "https://y.qq.com/n/ryqq/player")
            .header("Host", "c.y.qq.com")
            .header("Origin", "https://y.qq.com");

        let data: LyricsData = self.base.fetch_with_retry(request).await?;

        // 处理Base64解码
        let decoded = BASE64_STANDARD
            .decode(data.lyric)
            .map_err(|_| LyricsError::LyricsDecodeError)?;

        let re = String::from_utf8(decoded).map_err(|_| LyricsError::LyricsDecodeError)?;
        if re.is_empty() {
            return Err(LyricsError::NoLyricsFound);
        }
        Ok(re)
    }

    fn source_name(&self) -> &'static str {
        "QQMusic"
    }
}
