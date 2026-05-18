use serde::{Deserialize, Serialize};

pub const HEARTBEAT_TTL_SECS: i64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum LobbyStatus {
    Waiting,
    InGame,
    Finished,
}

impl LobbyStatus {
    pub fn to_str(&self) -> &'static str {
        match self {
            LobbyStatus::Waiting => "waiting",
            LobbyStatus::InGame => "in-game",
            LobbyStatus::Finished => "finished",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "waiting" => Some(LobbyStatus::Waiting),
            "in-game" => Some(LobbyStatus::InGame),
            "finished" => Some(LobbyStatus::Finished),
            _ => None,
        }
    }
}

pub struct LobbyPatch {
    pub allow_guests: Option<bool>,
    pub status: Option<LobbyStatus>,
    pub player_count: Option<u32>,
}
