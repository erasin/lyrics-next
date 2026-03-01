use std::time::Instant;

use crate::error::LyricsError;

pub use crate::config::PlayerProtocol;
pub use crate::player::{
    PlayerAction, SongInfo, TrackId, get_current_song, get_position, player_action,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PlayTime {
    pub current_time: f64,
    pub last_valid_pos: Option<(Instant, f64)>,
}

pub async fn get_current_time_song(st: PlayTime) -> Result<PlayTime, LyricsError> {
    let mut st = st;

    match get_position().await {
        Ok(pos) => {
            st.current_time = pos;
            st.last_valid_pos = Some((Instant::now(), pos));
        }
        Err(_) => {
            if let Some((time, pos)) = st.last_valid_pos {
                let delta = Instant::now().duration_since(time).as_secs_f64();
                st.current_time = pos + delta;
            }
        }
    }

    Ok(st)
}

#[derive(Debug, Clone)]
pub struct LyricsLine {
    pub timestamp_start: f64,
    pub timestamp_end: f64,
    pub text: String,
}

pub struct LyricParser;

impl LyricParser {
    pub async fn parse(doc: String, song_duration: f64) -> Result<Vec<LyricsLine>, LyricsError> {
        let mut entries = Vec::new();

        for line in doc.lines() {
            if let Ok((time_tags, text)) = Self::parse_line(line).await {
                for ts in time_tags {
                    entries.push((ts, text.clone()));
                }
            };
        }

        entries.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

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

        while line.starts_with('[') {
            let Some(end_idx) = line.find(']') else {
                break;
            };

            let time_str = &line[1..end_idx];
            line = &line[end_idx + 1..];

            match Self::parse_time(time_str).await {
                Some(time) => time_tags.push(time),
                None => return Err(LyricsError::InvalidTimeFormat),
            }
        }

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
