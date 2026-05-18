use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyMember {
    pub user_id: Uuid,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LobbyStatus {
    Waiting,
    InGame,
}

#[derive(Debug, Clone)]
pub struct Lobby {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub members: Vec<LobbyMember>,
    pub status: LobbyStatus,
    pub script_url: String,
    pub created_at: Instant,
}

impl Lobby {
    pub fn new(
        host_user_id: Uuid,
        host_display_name: String,
        script_url: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            host_user_id,
            members: vec![LobbyMember {
                user_id: host_user_id,
                display_name: host_display_name,
            }],
            status: LobbyStatus::Waiting,
            script_url,
            created_at: Instant::now(),
        }
    }

    pub fn member_user_ids(&self) -> Vec<Uuid> {
        self.members.iter().map(|m| m.user_id).collect()
    }
}

#[derive(Debug)]
pub enum JoinError {
    NotFound,
    GameAlreadyStarted,
}

#[derive(Debug)]
pub enum StartGameError {
    NotFound,
    AlreadyStarted,
    NotEnoughPlayers,
}

#[derive(Default)]
pub struct LobbyRegistry(DashMap<Uuid, Lobby>);

impl LobbyRegistry {
    pub fn create(
        &self,
        host_user_id: Uuid,
        host_display_name: String,
        script_url: String,
    ) -> Lobby {
        let lobby = Lobby::new(host_user_id, host_display_name, script_url);
        self.0.insert(lobby.id, lobby.clone());
        lobby
    }

    pub fn get(&self, id: &Uuid) -> Option<Lobby> {
        self.0.get(id).map(|l| l.clone())
    }

    pub fn join(
        &self,
        lobby_id: &Uuid,
        user_id: Uuid,
        display_name: String,
    ) -> Result<(LobbyMember, Vec<Uuid>), JoinError> {
        let mut entry = self.0.get_mut(lobby_id).ok_or(JoinError::NotFound)?;
        if entry.status != LobbyStatus::Waiting {
            return Err(JoinError::GameAlreadyStarted);
        }
        if entry.members.iter().any(|m| m.user_id == user_id) {
            let member = entry
                .members
                .iter()
                .find(|m| m.user_id == user_id)
                .unwrap()
                .clone();
            let ids = entry.member_user_ids();
            return Ok((member, ids));
        }
        let member = LobbyMember { user_id, display_name };
        entry.members.push(member.clone());
        let ids = entry.member_user_ids();
        Ok((member, ids))
    }

    /// Remove a member. Returns (remaining_member_ids, was_host).
    pub fn leave(&self, lobby_id: &Uuid, user_id: &Uuid) -> Option<(Vec<Uuid>, bool)> {
        let mut entry = self.0.get_mut(lobby_id)?;
        let was_host = entry.host_user_id == *user_id;
        entry.members.retain(|m| m.user_id != *user_id);
        let ids = entry.member_user_ids();
        Some((ids, was_host))
    }

    pub fn delete(&self, lobby_id: &Uuid) {
        self.0.remove(lobby_id);
    }

    pub fn start_game(&self, lobby_id: &Uuid) -> Result<Vec<LobbyMember>, StartGameError> {
        let mut entry = self.0.get_mut(lobby_id).ok_or(StartGameError::NotFound)?;
        if entry.status != LobbyStatus::Waiting {
            return Err(StartGameError::AlreadyStarted);
        }
        if entry.members.len() < 2 {
            return Err(StartGameError::NotEnoughPlayers);
        }
        entry.status = LobbyStatus::InGame;
        Ok(entry.members.clone())
    }

    pub fn set_game_ended(&self, lobby_id: &Uuid) -> Option<Vec<Uuid>> {
        let mut entry = self.0.get_mut(lobby_id)?;
        entry.status = LobbyStatus::Waiting;
        Some(entry.member_user_ids())
    }
}
