use std::time::{Duration, Instant};

use anyhow::Context;
use mpris::{Player, PlayerFinder, TrackID};

use crate::{config::get_config, error::LyricsError, utils::normalize_text};

/// 歌曲信息
#[derive(Debug, Clone, PartialEq)]
pub struct SongInfo {
    pub track_id: TrackID,
    /// 标题
    pub title: String,
    /// 作者
    pub artist: String,
    pub album: String,
    /// 时长
    pub duration: f64,
}

impl Default for SongInfo {
    fn default() -> Self {
        Self {
            track_id: TrackID::no_track(),
            title: Default::default(),
            artist: Default::default(),
            album: Default::default(),
            duration: Default::default(),
        }
    }
}

impl SongInfo {
    /// 歌曲信息的标准化
    #[allow(dead_code)]
    fn normalized(&self) -> Self {
        Self {
            title: normalize_text(&self.title),
            artist: normalize_text(&self.artist),
            duration: 0.,
            ..self.clone()
        }
    }

    pub(crate) fn track_support(&self) -> bool {
        self.track_id == TrackID::no_track()
    }
}

/// 过滤白名单和黑名单
fn is_valid_player(player: &Player) -> bool {
    let identity = player.identity().to_lowercase();

    let config = &get_config().read().unwrap().player_filter;

    if !config.except.is_empty() && config.except.iter().any(|k| identity.contains(k)) {
        return false;
    }

    if !config.only.is_empty() {
        return config.only.iter().any(|k| identity.contains(k));
    }

    true
}

/// 获取 当前播放的 mpris player
async fn get_player() -> Result<Player, LyricsError> {
    let player_finder = PlayerFinder::new()?;
    let player = player_finder
        .find_all()?
        .into_iter()
        .filter(is_valid_player)
        .max_by_key(|p| p.is_running()) // 优先选择正在播放的
        .ok_or_else(|| LyricsError::NoPlayerFound)?;

    Ok(player)
}

/// 获取当前播放歌曲
pub async fn get_current_song() -> Result<SongInfo, LyricsError> {
    let player = get_player().await?;
    let metadata = player.get_metadata()?;

    // track_id 有些不支持
    let track_id = metadata.track_id().unwrap_or(TrackID::no_track());

    // 获取所有活动播放器并过滤
    let title = metadata.title().context("无标题")?.to_string();
    let artist = metadata.artists().map(|a| a.join(", ")).context("无作家")?;
    let album = metadata.album_name().unwrap_or_default().to_string();
    let duration = metadata.length().map(|d| d.as_secs_f64()).unwrap_or(0.0);

    Ok(SongInfo {
        track_id,
        title,
        artist,
        album,
        duration,
    })
}

/// 播放时间
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayTime {
    /// 当前时间
    pub current_time: f64,
    /// 最后校正时间
    pub last_valid_pos: Option<(Instant, f64)>,
}

/// 获取当前歌曲的播放时间
pub async fn get_current_time_song(st: PlayTime) -> Result<PlayTime, LyricsError> {
    let player = get_player().await?;
    let mut st = st;

    match player.get_position().map(|d| d.as_secs_f64()) {
        Ok(pos) => {
            st.current_time = pos;
            st.last_valid_pos = Some((Instant::now(), pos));
        }
        Err(_) => {
            // 根据最后一次有效位置和流逝时间估算
            if let Some((time, pos)) = st.last_valid_pos {
                let delta = Instant::now().duration_since(time).as_secs_f64();
                st.current_time = pos + delta;
            }
        }
    }

    Ok(st)
}

/// 播放控制
#[derive(Clone, PartialEq, Eq, Default)]
pub enum PlayerAction {
    #[default]
    Toggle,
    /// 后退
    Left,
    /// 前进
    Right,
    /// 下一首
    Next,
    /// 上一首
    Previous,
}

pub async fn player_action(action: PlayerAction, song: &SongInfo) -> Result<(), LyricsError> {
    let player = get_player().await?;

    match action {
        PlayerAction::Toggle => player.play_pause()?,
        PlayerAction::Left => {
            if song.track_support() {
                return Ok(());
            }
            let add = Duration::from_secs(5);
            let pos = player.get_position()? - add;
            player.set_position(song.track_id.clone(), &pos)?;
        }
        PlayerAction::Right => {
            if song.track_support() {
                return Ok(());
            }
            let add = Duration::from_secs(5);
            let pos = player.get_position()? + add;
            player.set_position(song.track_id.clone(), &pos)?;
        }
        PlayerAction::Next => player.next()?,
        PlayerAction::Previous => player.previous()?,
    }

    Ok(())
}

/// 歌词行
#[derive(Debug, Clone)]
pub struct LyricsLine {
    pub timestamp_start: f64, // 单位：秒
    pub timestamp_end: f64,   // 单位：秒
    pub text: String,
}

// 解析主逻辑
pub struct LyricParser;

impl LyricParser {
    pub async fn parse(doc: String, song_duration: f64) -> Result<Vec<LyricsLine>, LyricsError> {
        let mut entries = Vec::new();

        // 第一阶段：收集所有时间标签和文本
        for line in doc.lines() {
            // let line_str = line.to_string();
            if let Ok((time_tags, text)) = Self::parse_line(line).await {
                for ts in time_tags {
                    entries.push((ts, text.clone()));
                }
            };
        }

        // 按时间排序
        entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        // 第二阶段：创建带时间区间的歌词行
        let mut lyrics = Vec::with_capacity(entries.len());
        for (i, &(start, ref text)) in entries.iter().enumerate() {
            let end = entries
                .get(i + 1)
                .map(|(next_start, _)| *next_start)
                .unwrap_or(song_duration);

            lyrics.push(LyricsLine {
                timestamp_start: start,
                timestamp_end: end,
                text: text.clone(),
            });
        }

        if lyrics.is_empty() {
            Err(LyricsError::EmptyLyrics)
        } else {
            Ok(lyrics)
        }
    }

    async fn parse_line(line: &str) -> Result<(Vec<f64>, String), LyricsError> {
        let mut line = line.trim();
        let mut time_tags = Vec::new();

        // 解析时间标签
        while line.starts_with('[') {
            let Some(end_idx) = line.find(']') else {
                break;
            };

            let time_str = &line[1..end_idx];
            // 余下为内容
            line = &line[end_idx + 1..];

            match Self::parse_time(time_str).await {
                Some(time) => time_tags.push(time),
                None => return Err(LyricsError::InvalidTimeFormat),
            }
        }

        // 添加有效歌词行
        let text = line.trim().to_string();

        Ok((time_tags, text))
    }

    async fn parse_time(s: &str) -> Option<f64> {
        let parts: Vec<&str> = s.split(&[':', '.']).collect();
        if parts.len() < 2 {
            return None;
        }

        let minutes = parts[0].parse::<f64>().ok()?;
        let seconds = parts[1].parse::<f64>().ok()?;
        let millis = parts
            .get(2)
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        Some(minutes * 60.0 + seconds + millis / 100.0)
    }
}
