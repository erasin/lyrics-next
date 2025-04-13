use async_trait::async_trait;
use serde::Deserialize;

use super::{BaseFetcher, LyricsFetcher};
use crate::{error::LyricsError, song::SongInfo};

#[derive(Debug, Deserialize)]
struct Response {
    result: SongResult,
    // code: u64,
}

// data
#[derive(Debug, Deserialize)]
struct SongResult {
    songs: Vec<Song>,
}

#[derive(Debug, Deserialize)]
struct Song {
    id: u64,
    name: String,
    artists: Vec<Artist>,
    album: Album,
    // duration: u64,
}

#[derive(Debug, Deserialize)]
struct Artist {
    // id: u64,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Album {
    // id: u64,
    name: String,
    // size: u64,
}

#[derive(Debug, Deserialize)]
struct LyricData {
    lrc: LrcData,
}

#[derive(Debug, Deserialize)]
struct LrcData {
    lyric: String,
}

// 网易云音乐实现
#[derive(Default)]
pub(super) struct NeteaseFetcher {
    base: BaseFetcher,
}

#[async_trait]
impl LyricsFetcher for NeteaseFetcher {
    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError> {
        log::debug!("Netease song: {:?}", song);
        let search_url = "https://music.163.com/api/search/get/";

        let request = self.base.client.get(search_url).query(&[
            // ("s", format!("{} {}", song.title, song.artist)),
            ("s", song.title.as_str()),
            ("type", "1"),
            ("limit", "10"), // song_id 1, album_id 10 playlist_id 1000
            ("offset", "0"),
        ]);

        let data = self.base.fetch_with_retry::<Response>(request).await?;

        log::debug!("Get song: {:?}, info: {:?}", data, song);

        let song_id = data
            .result
            .songs
            .into_iter()
            .filter(|s| s.name == song.title || s.name.contains(&song.title))
            .filter(|s| {
                if song.artist.is_empty() {
                    true
                } else {
                    s.artists.iter().find(|&a| a.name == song.artist).is_some()
                }
            })
            .filter(|s| {
                if song.album.is_empty() {
                    true
                } else {
                    let a = s.album.name.to_lowercase();
                    let b = song.album.to_lowercase();
                    a == b || a.contains(&b)
                }
            })
            .next()
            .map(|s| s.id)
            .ok_or(LyricsError::NoLyricsFound)?;

        log::debug!("Get song id: {:?}", song_id);

        let lyric_url = format!("https://music.163.com/api/song/lyric?id={}&lv=1", song_id);
        let request = self
            .base
            .client
            .get(lyric_url)
            .query(&[("id", song_id), ("lv", 1)]);

        let data: LyricData = self.base.fetch_with_retry(request).await?;

        log::debug!("Get lyric: {:?}", data);

        Ok(data.lrc.lyric)
    }

    fn source_name(&self) -> &'static str {
        "Netease"
    }
}
