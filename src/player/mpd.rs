use mpd::Client;
use std::time::Duration;
use tracing::debug;

use crate::{config::get_config, error::LyricsError};

use super::{Player, PlayerAction, SongInfo, TrackId};

pub struct MpdPlayer;

fn get_client() -> Result<Client, LyricsError> {
    let config = get_config().read().unwrap();
    let host = &config.player_filter.mpd_host;
    let port = config.player_filter.mpd_port;

    let addr = format!("{}:{}", host, port);
    debug!("连接到 MPD: {}", addr);

    let client = Client::connect(&addr)?;
    Ok(client)
}

impl Player for MpdPlayer {
    async fn get_current_song(&self) -> Result<SongInfo, LyricsError> {
        let mut client = get_client()?;
        let song = client.currentsong()?.ok_or(LyricsError::NoPlayerFound)?;

        let track_id = song.place.map(|p| TrackId::Mpd(p.id.0)).unwrap_or_default();

        let title = song.title.unwrap_or_default();
        let artist = song.artist.unwrap_or_default();
        let album = song
            .tags
            .iter()
            .find(|(k, _)| k == "Album")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let duration = song.duration.map(|d| d.as_secs_f64()).unwrap_or(0.0);

        Ok(SongInfo {
            track_id,
            title,
            artist,
            album,
            duration,
        })
    }

    async fn get_position(&self) -> Result<f64, LyricsError> {
        let mut client = get_client()?;
        let status = client.status()?;
        let pos = status.elapsed.map(|d| d.as_secs_f64()).unwrap_or(0.0);
        Ok(pos)
    }

    async fn player_action(
        &self,
        action: PlayerAction,
        song: &SongInfo,
    ) -> Result<(), LyricsError> {
        let mut client = get_client()?;

        match action {
            PlayerAction::Toggle => {
                let status = client.status()?;
                match status.state {
                    mpd::State::Play => client.pause(true)?,
                    mpd::State::Pause => client.pause(false)?,
                    _ => {}
                }
            }
            PlayerAction::Left => {
                if song.track_support() {
                    return Ok(());
                }
                let status = client.status()?;
                let current = status.elapsed.unwrap_or(Duration::from_secs(0));
                let new_pos = current.saturating_sub(Duration::from_secs(5));
                if let TrackId::Mpd(id) = song.track_id {
                    client.seek(mpd::Id(id), new_pos)?;
                }
            }
            PlayerAction::Right => {
                if song.track_support() {
                    return Ok(());
                }
                let status = client.status()?;
                let current = status.elapsed.unwrap_or(Duration::from_secs(0));
                let new_pos = current.saturating_add(Duration::from_secs(5));
                if let TrackId::Mpd(id) = song.track_id {
                    client.seek(mpd::Id(id), new_pos)?;
                }
            }
            PlayerAction::Next => client.next()?,
            PlayerAction::Previous => client.prev()?,
        }

        Ok(())
    }
}
