use async_trait::async_trait;
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::Deserialize;
use tracing::debug;

use super::{BaseFetcher, LyricsFetcher, LyricsItem};
use crate::{client::get_first, error::LyricsError, song::SongInfo};

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
        debug!("Get song: {:?}, info: {:?}", data, song);

        let list: Vec<LyricsItem> = data
            .data
            .song
            .list
            .into_iter()
            .map(|s| {
                let source = self.source_name().into();
                let title = s.songname;
                let artist = s
                    .singer
                    .iter()
                    .map(|aa| aa.name.clone())
                    .collect::<Vec<String>>()
                    .join(" ");
                let album = s.albumname;
                let params = vec![("songmid".to_string(), s.songmid)];

                LyricsItem {
                    source,
                    title,
                    artist,
                    album,
                    params,
                }
            })
            .collect();

        debug!("Get List: {:?}", list);

        if !list.is_empty() {
            debug!("Get List send",);
            Ok(list)
        } else {
            Err(LyricsError::NoLyricsFound)
        }
    }

    async fn download_lyric(&self, item: &LyricsItem) -> Result<String, LyricsError> {
        let mut params = item.params.clone();
        params.append(&mut vec![
            ("format".to_string(), "json".to_string()),
            ("g_tk".to_string(), "5381".to_string()),
        ]);

        // 2. 获取歌词
        let lyrics_url = "https://c.y.qq.com/lyric/fcgi-bin/fcg_query_lyric_new.fcg";
        let request = self
            .base
            .client
            .get(lyrics_url)
            .query(&params)
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
    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError> {
        debug!("QQ search");

        // let song_mid = data
        //     .data
        //     .song
        //     .list
        //     .into_iter()
        //     .filter(|s| s.songname == song.title || s.songname.contains(&song.title))
        //     .filter(|s| {
        //         if song.artist.is_empty() {
        //             true
        //         } else {
        //             s.singer.iter().find(|&a| a.name == song.artist).is_some()
        //         }
        //     })
        //     .filter(|s| {
        //         if song.album.is_empty() {
        //             true
        //         } else {
        //             let a = s.albumname.to_lowercase();
        //             let b = song.album.to_lowercase();
        //             a == b || a.contains(&b)
        //         }
        //     })
        //     .next()
        //     .map(|s| s.songmid)
        //     .ok_or(LyricsError::NoLyricsFound)?;

        // debug!("song mid : {song_mid}");

        let list = self.search_lyric(song).await?;
        let item = get_first(list, song)?;
        debug!("Get song: {:?} info: {:?}", item, song);
        self.download_lyric(&item).await
    }

    fn source_name(&self) -> &'static str {
        "QQMusic"
    }
}
