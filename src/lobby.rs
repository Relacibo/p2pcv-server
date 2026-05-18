use dashmap::DashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub const HEARTBEAT_TTL: Duration = Duration::from_secs(90);

#[derive(Debug, Clone)]
pub struct Lobby {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub script_url: String,
    pub allow_guests: bool,
    pub created_at: Instant,
    pub last_heartbeat: Instant,
}

impl Lobby {
    pub fn new(host_user_id: Uuid, script_url: String, allow_guests: bool) -> Self {
        let now = Instant::now();
        Self {
            id: Uuid::new_v4(),
            host_user_id,
            script_url,
            allow_guests,
            created_at: now,
            last_heartbeat: now,
        }
    }
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

    /// Update host after migration. Returns false if lobby not found.
    pub fn update_host(&self, lobby_id: &Uuid, new_host_user_id: Uuid) -> bool {
        if let Some(mut entry) = self.0.get_mut(lobby_id) {
            entry.host_user_id = new_host_user_id;
            entry.last_heartbeat = Instant::now();
            return true;
        }
        false
    }

        pub fn update_settings(&self, lobby_id: &Uuid, allow_guests: bool) -> bool {
        if let Some(mut entry) = self.0.get_mut(lobby_id) {
            entry.allow_guests = allow_guests;
            return true;
        }
        false
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
