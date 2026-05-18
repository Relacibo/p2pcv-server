use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::{
    app_result::AppResult,
    lobby::{LobbyPatch, LobbyStatus},
};

use super::entities::lobbies as lobby;

pub type Lobby = lobby::Model;

pub struct NewLobby {
    pub host_user_id: Uuid,
    pub script_url: String,
    pub allow_guests: bool,
}

pub struct LobbyListParams {
    pub page: u64,
    pub limit: u64,
    pub allow_guests: Option<bool>,
    pub status: Option<LobbyStatus>,
    pub script_url: Option<String>,
}

pub struct LobbyPage {
    pub items: Vec<Lobby>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

impl lobby::Model {
    pub async fn create(db: &DatabaseConnection, new: NewLobby) -> AppResult<Lobby> {
        let now = Utc::now();
        let model = lobby::ActiveModel {
            id: Set(Uuid::new_v4()),
            host_user_id: Set(new.host_user_id),
            host_peer_session_id: Set(None),
            script_url: Set(new.script_url),
            allow_guests: Set(new.allow_guests),
            status: Set(LobbyStatus::Waiting.to_str().to_owned()),
            player_count: Set(1),
            min_players: Set(None),
            max_players: Set(None),
            last_heartbeat: Set(now.into()),
            created_at: Set(now.into()),
        };
        let lobby = model.insert(db).await?;
        Ok(lobby)
    }

    pub async fn get(db: &DatabaseConnection, id: Uuid) -> AppResult<Option<Lobby>> {
        Ok(lobby::Entity::find_by_id(id).one(db).await?)
    }

    pub async fn list(db: &DatabaseConnection, params: LobbyListParams) -> AppResult<LobbyPage> {
        let limit = params.limit.clamp(1, 100);
        let mut query = lobby::Entity::find()
            .order_by_desc(lobby::Column::CreatedAt);

        if let Some(allow_guests) = params.allow_guests {
            query = query.filter(lobby::Column::AllowGuests.eq(allow_guests));
        }
        if let Some(status) = params.status {
            query = query.filter(lobby::Column::Status.eq(status.to_str()));
        }
        if let Some(script_url) = params.script_url.as_deref().filter(|s| !s.is_empty()) {
            query = query.filter(lobby::Column::ScriptUrl.eq(script_url));
        }

        let paginator = query.paginate(db, limit);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(params.page).await?;

        Ok(LobbyPage {
            items,
            total,
            page: params.page,
            limit,
        })
    }

    pub async fn patch(
        db: &DatabaseConnection,
        id: Uuid,
        host_user_id: Uuid,
        patch: LobbyPatch,
    ) -> AppResult<Option<bool>> {
        let Some(record) = lobby::Entity::find_by_id(id).one(db).await? else {
            return Ok(None);
        };
        if record.host_user_id != host_user_id {
            return Ok(Some(false));
        }
        let mut active: lobby::ActiveModel = record.into();
        if let Some(v) = patch.allow_guests {
            active.allow_guests = Set(v);
        }
        if let Some(v) = patch.status {
            active.status = Set(v.to_str().to_owned());
        }
        if let Some(v) = patch.player_count {
            active.player_count = Set(v as i32);
        }
        active.update(db).await?;
        Ok(Some(true))
    }

    pub async fn patch_peer_session_id(
        db: &DatabaseConnection,
        id: Uuid,
        host_user_id: Uuid,
        peer_session_id: Option<String>,
    ) -> AppResult<Option<bool>> {
        let Some(record) = lobby::Entity::find_by_id(id).one(db).await? else {
            return Ok(None);
        };
        if record.host_user_id != host_user_id {
            return Ok(Some(false));
        }
        let mut active: lobby::ActiveModel = record.into();
        active.host_peer_session_id = Set(peer_session_id);
        active.update(db).await?;
        Ok(Some(true))
    }

    /// Returns `false` if the lobby was not found or the caller is not the host.
    pub async fn heartbeat(
        db: &DatabaseConnection,
        id: Uuid,
        host_user_id: Uuid,
    ) -> AppResult<bool> {
        let Some(record) = lobby::Entity::find_by_id(id).one(db).await? else {
            return Ok(false);
        };
        if record.host_user_id != host_user_id {
            return Ok(false);
        }
        let mut active: lobby::ActiveModel = record.into();
        active.last_heartbeat = Set(Utc::now().into());
        active.update(db).await?;
        Ok(true)
    }

    pub async fn delete(
        db: &DatabaseConnection,
        id: Uuid,
        host_user_id: Uuid,
    ) -> AppResult<Option<bool>> {
        let Some(record) = lobby::Entity::find_by_id(id).one(db).await? else {
            return Ok(None);
        };
        if record.host_user_id != host_user_id {
            return Ok(Some(false));
        }
        lobby::Entity::delete_by_id(id).exec(db).await?;
        Ok(Some(true))
    }

    /// Deletes all lobbies whose last heartbeat is older than `ttl_seconds`. Returns deleted IDs.
    pub async fn remove_stale(
        db: &DatabaseConnection,
        ttl_seconds: i64,
    ) -> AppResult<Vec<Uuid>> {
        let cutoff = Utc::now() - chrono::Duration::seconds(ttl_seconds);
        let stale = lobby::Entity::find()
            .filter(lobby::Column::LastHeartbeat.lt(cutoff.fixed_offset()))
            .all(db)
            .await?;
        let ids: Vec<Uuid> = stale.iter().map(|l| l.id).collect();
        if !ids.is_empty() {
            lobby::Entity::delete_many()
                .filter(lobby::Column::Id.is_in(ids.clone()))
                .exec(db)
                .await?;
        }
        Ok(ids)
    }
}
