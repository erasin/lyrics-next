use async_trait::async_trait;
use serde::Deserialize;
use tracing::debug;

use super::{BaseFetcher, LyricsFetcher, LyricsItem};
use crate::{client::get_first, error::LyricsError, song::SongInfo};

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

impl NeteaseFetcher {}

#[async_trait]
impl LyricsFetcher for NeteaseFetcher {
    async fn search_lyric(&self, song: &SongInfo) -> Result<Vec<LyricsItem>, LyricsError> {
        let search_url = "https://music.163.com/api/search/get/";
        // TODO 分词处理

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
                let artist = s
                    .artists
                    .iter()
                    .map(|aa| aa.name.clone())
                    .collect::<Vec<String>>()
                    .join(" ");
                // .fold(String::new(), |mut full, a| {
                //     full.push_str(&a.name);
                //     full
                // });
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

        debug!("Get List: {:?}", list);

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
        debug!("Get lyric: {:?}", data);
        Ok(data.lrc.lyric)
    }

    async fn fetch_lyric(&self, song: &SongInfo) -> Result<String, LyricsError> {
        debug!("Netease song: {:?}", song);
        let list = self.search_lyric(song).await?;
        let item = get_first(list, song)?;
        debug!("Get song: {:?} info: {:?}", item, song);
        self.download_lyric(&item).await
    }

    fn source_name(&self) -> &'static str {
        "Netease"
    }
}
