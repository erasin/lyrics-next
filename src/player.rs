mod mpd;
mod mpris;

pub use mpd::MpdPlayer;
pub use mpris::MprisPlayer;

use crate::error::LyricsError;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TrackId {
    Mpris(String),
    Mpd(u32),
    #[default]
    None,
}

impl TrackId {
    pub fn is_no_track(&self) -> bool {
        matches!(self, TrackId::None)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SongInfo {
    pub track_id: TrackId,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
}

impl Default for SongInfo {
    fn default() -> Self {
        Self {
            track_id: TrackId::None,
            title: Default::default(),
            artist: Default::default(),
            album: Default::default(),
            duration: Default::default(),
        }
    }
}

impl SongInfo {
    #[allow(dead_code)]
    fn normalized(&self) -> Self {
        use crate::utils::normalize_text;
        Self {
            title: normalize_text(&self.title),
            artist: normalize_text(&self.artist),
            duration: 0.,
            ..self.clone()
        }
    }

    pub(crate) fn track_support(&self) -> bool {
        self.track_id.is_no_track()
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub enum PlayerAction {
    #[default]
    Toggle,
    Left,
    Right,
    Next,
    Previous,
}

pub trait Player {
    fn get_current_song(&self) -> impl std::future::Future<Output = Result<SongInfo, LyricsError>>;
    fn get_position(&self) -> impl std::future::Future<Output = Result<f64, LyricsError>>;
    fn player_action(
        &self,
        action: PlayerAction,
        song: &SongInfo,
    ) -> impl std::future::Future<Output = Result<(), LyricsError>>;
}

pub async fn get_current_song() -> Result<SongInfo, LyricsError> {
    let protocol = {
        let config = crate::config::get_config().read().unwrap();
        config.player_filter.protocol
    };

    match protocol {
        crate::config::PlayerProtocol::Auto => {
            // 优先尝试 MPD
            match MpdPlayer.get_current_song().await {
                Ok(song) => Ok(song),
                Err(_) => {
                    // MPD 失败，回退到 MPRIS
                    MprisPlayer.get_current_song().await
                }
            }
        }
        crate::config::PlayerProtocol::Mpd => MpdPlayer.get_current_song().await,
        crate::config::PlayerProtocol::Mpris => MprisPlayer.get_current_song().await,
    }
}

pub async fn get_position() -> Result<f64, LyricsError> {
    let protocol = {
        let config = crate::config::get_config().read().unwrap();
        config.player_filter.protocol
    };

    match protocol {
        crate::config::PlayerProtocol::Auto => {
            // 优先尝试 MPD
            match MpdPlayer.get_position().await {
                Ok(pos) => Ok(pos),
                Err(_) => {
                    // MPD 失败，回退到 MPRIS
                    MprisPlayer.get_position().await
                }
            }
        }
        crate::config::PlayerProtocol::Mpd => MpdPlayer.get_position().await,
        crate::config::PlayerProtocol::Mpris => MprisPlayer.get_position().await,
    }
}

pub async fn player_action(action: PlayerAction, song: &SongInfo) -> Result<(), LyricsError> {
    let protocol = {
        let config = crate::config::get_config().read().unwrap();
        config.player_filter.protocol
    };

    match protocol {
        crate::config::PlayerProtocol::Auto => {
            // 优先尝试 MPD
            match MpdPlayer.player_action(action.clone(), song).await {
                Ok(()) => Ok(()),
                Err(_) => {
                    // MPD 失败，回退到 MPRIS
                    MprisPlayer.player_action(action, song).await
                }
            }
        }
        crate::config::PlayerProtocol::Mpd => MpdPlayer.player_action(action, song).await,
        crate::config::PlayerProtocol::Mpris => MprisPlayer.player_action(action, song).await,
    }
}
