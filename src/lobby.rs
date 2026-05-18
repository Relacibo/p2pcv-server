use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const HEARTBEAT_TTL: Duration = Duration::from_secs(300);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum LobbyStatus {
    Waiting,
    InGame,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Lobby {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub host_peer_session_id: Option<String>,
    pub script_url: String,
    pub allow_guests: bool,
    pub status: LobbyStatus,
    pub player_count: u32,
    pub min_players: Option<u32>,
    pub max_players: Option<u32>,
    pub created_at: Instant,
    pub last_heartbeat: Instant,
}

impl Lobby {
    pub fn new(host_user_id: Uuid, script_url: String, allow_guests: bool) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4(),
            host_user_id,
            host_peer_session_id: None,
            script_url,
            allow_guests,
            status: LobbyStatus::Waiting,
            player_count: 1,
            min_players: None,
            max_players: None,
            created_at: now,
            last_heartbeat: now,
        }
    }
}

pub struct LobbyPatch {
    pub allow_guests: Option<bool>,
    pub status: Option<LobbyStatus>,
    pub player_count: Option<u32>,
    pub min_players: Option<Option<u32>>,
    pub max_players: Option<Option<u32>>,
}

#[derive(Default)]
pub struct LobbyRegistry(DashMap<Uuid, Lobby>);

impl LobbyRegistry {
    pub fn create(&self, host_user_id: Uuid, script_url: String, allow_guests: bool) -> Lobby {
        let lobby = Lobby::new(host_user_id, script_url, allow_guests);
        self.0.insert(lobby.id, lobby.clone());
        lobby
    }

    pub fn get(&self, id: &Uuid) -> Option<Lobby> {
        self.0.get(id).map(|l| l.clone())
    }

    pub fn delete(&self, lobby_id: &Uuid) {
        self.0.remove(lobby_id);
    }

    /// Returns false if lobby not found or caller is not the host.
    pub fn heartbeat(&self, lobby_id: &Uuid, host_user_id: &Uuid) -> bool {
        if let Some(mut entry) = self.0.get_mut(lobby_id) {
            if entry.host_user_id == *host_user_id {
                entry.last_heartbeat = Instant::now();
                return true;
            }
        }
        false
    }

    /// Apply a partial update. Returns `None` if not found, `Some(false)` if not the host.
    pub fn patch(&self, lobby_id: &Uuid, host_user_id: &Uuid, patch: LobbyPatch) -> Option<bool> {
        let mut entry = self.0.get_mut(lobby_id)?;
        if entry.host_user_id != *host_user_id {
            return Some(false);
        }
        if let Some(v) = patch.allow_guests { entry.allow_guests = v; }
        if let Some(v) = patch.status { entry.status = v; }
        if let Some(v) = patch.player_count { entry.player_count = v; }
        if let Some(v) = patch.min_players { entry.min_players = v; }
        if let Some(v) = patch.max_players { entry.max_players = v; }
        Some(true)
    }

    /// Remove lobbies whose last heartbeat is older than `ttl`.
    /// Returns the IDs of deleted lobbies.
    pub fn remove_stale(&self, ttl: Duration) -> Vec<Uuid> {
        let stale: Vec<Uuid> = self
            .0
            .iter()
            .filter(|entry| entry.last_heartbeat.elapsed() > ttl)
            .map(|entry| *entry.key())
            .collect();
        for id in &stale {
            self.0.remove(id);
        }
        stale
    }
}
