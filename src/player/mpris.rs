use std::time::Duration;

use anyhow::Context;
use mpris::{Player as MprisClient, PlayerFinder, TrackID};

use crate::{config::get_config, error::LyricsError};

use super::{Player, PlayerAction, SongInfo, TrackId};

pub struct MprisPlayer;

fn is_valid_player(player: &MprisClient) -> bool {
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

async fn get_player() -> Result<MprisClient, LyricsError> {
    let player_finder = PlayerFinder::new()?;
    let player = player_finder
        .find_all()?
        .into_iter()
        .filter(is_valid_player)
        .max_by_key(|p| p.is_running())
        .ok_or_else(|| LyricsError::NoPlayerFound)?;

    Ok(player)
}

impl Player for MprisPlayer {
    async fn get_current_song(&self) -> Result<SongInfo, LyricsError> {
        let player = get_player().await?;
        let metadata = player.get_metadata()?;

        let track_id = metadata
            .track_id()
            .map(|tid| TrackId::Mpris(tid.to_string()))
            .unwrap_or_default();

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

    async fn get_position(&self) -> Result<f64, LyricsError> {
        let player = get_player().await?;
        let pos = player.get_position().map(|d| d.as_secs_f64())?;
        Ok(pos)
    }

    async fn player_action(
        &self,
        action: PlayerAction,
        song: &SongInfo,
    ) -> Result<(), LyricsError> {
        let player = get_player().await?;

        match action {
            PlayerAction::Toggle => player.play_pause()?,
            PlayerAction::Left => {
                if song.track_support() {
                    return Ok(());
                }
                let add = Duration::from_secs(5);
                let pos = player.get_position()? - add;
                if let TrackId::Mpris(ref tid_str) = song.track_id {
                    let track_id =
                        TrackID::new(tid_str.clone()).map_err(|e| anyhow::anyhow!("{}", e))?;
                    player.set_position(track_id, &pos)?;
                }
            }
            PlayerAction::Right => {
                if song.track_support() {
                    return Ok(());
                }
                let add = Duration::from_secs(5);
                let pos = player.get_position()? + add;
                if let TrackId::Mpris(ref tid_str) = song.track_id {
                    let track_id =
                        TrackID::new(tid_str.clone()).map_err(|e| anyhow::anyhow!("{}", e))?;
                    player.set_position(track_id, &pos)?;
                }
            }
            PlayerAction::Next => player.next()?,
            PlayerAction::Previous => player.previous()?,
        }

        Ok(())
    }
}
