use async_trait::async_trait;
use serde::Deserialize;

use super::{BaseFetcher, LyricsFetcher, LyricsItem};
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

impl NeteaseFetcher {
    fn get_first(&self, list: Vec<LyricsItem>, song: &SongInfo) -> Result<LyricsItem, LyricsError> {
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
                        s.artist == song.artist || s.artist.contains(&song.artist)
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
}

#[async_trait]
impl LyricsFetcher for NeteaseFetcher {
    async fn search_lyric(&self, song: &SongInfo) -> Result<Vec<LyricsItem>, LyricsError> {
        let search_url = "https://music.163.com/api/search/get/";

        let request = self.base.client.get(search_url).query(&[
            ("s", format!("{} {}", song.title, song.artist).as_str()),
            ("type", "1"),
            ("limit", "10"), // song_id 1, album_id 10 playlist_id 1000
            ("offset", "0"),
        ]);

        let data = self.base.fetch_with_retry::<Response>(request).await?;

        let list: Vec<LyricsItem> = data
            .result
            .songs
            .into_iter()
            .map(|s| {
                let source = self.source_name().into();
                let title = s.name;
                let artist = s.artists.iter().fold(String::new(), |mut full, a| {
                    full.push_str(&a.name);
                    full
                });
                let album = s.album.name;
                let params = vec![
                    ("id".to_string(), s.id.to_string()),
                    ("lv".to_string(), "1".to_string()),
                ];

                LyricsItem {
                    source,
                    title,
                    artist,
                    album,
                    params,
                }
            })
            .collect();

        log::debug!("Get List: {:?}", list);

        if !list.is_empty() {
            Ok(list)
        } else {
            Err(LyricsError::NoLyricsFound)
        }
    }

    async fn download_lyric(&self, item: &LyricsItem) -> Result<String, LyricsError> {
        let lyric_url = "https://music.163.com/api/song/lyric";
        let request = self.base.client.get(lyric_url).query(&item.params);
        let data: LyricData = self.base.fetch_with_retry(request).await?;
        log::debug!("Get lyric: {:?}", data);
        Ok(data.lrc.lyric)
    }

    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError> {
        log::debug!("Netease song: {:?}", song);
        let list = self.search_lyric(song).await?;
        let item = self.get_first(list, song)?;
        log::debug!("Get song: {:?} info: {:?}", item, song);
        self.download_lyric(&item).await
    }

    fn source_name(&self) -> &'static str {
        "Netease"
    }
}
