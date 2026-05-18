use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState,
    api::auth::session::auth::Auth,
    db::lobbies::{Lobby, LobbyListParams, LobbyPage, NewLobby},
    error::AppError,
    lobby::{LobbyPatch, LobbyStatus},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_lobbies))
        .route("/", post(create_lobby))
        .route("/{id}", get(get_lobby))
        .route("/{id}", patch(patch_lobby))
        .route("/{id}", delete(delete_lobby))
        .route("/{id}/heartbeat", post(heartbeat))
        .route("/{id}/signal", post(relay_signal_in_lobby))
}

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLobbiesQuery {
    #[serde(default)]
    pub page: u64,
    #[serde(default = "default_limit")]
    pub limit: u64,
    pub allow_guests: Option<bool>,
    pub status: Option<String>,
    pub script_url: Option<String>,
}

fn default_limit() -> u64 {
    20
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListLobbiesResponse {
    pub items: Vec<LobbyResponse>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyPayload {
    pub script_url: String,
    pub allow_guests: bool,
    pub host_peer_session_id: String,
    pub min_players: i32,
    pub max_players: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyResponse {
    pub lobby_id: Uuid,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyResponse {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub host_peer_session_id: Option<String>,
    pub script_url: String,
    pub allow_guests: bool,
    pub status: String,
    pub player_count: i32,
    pub min_players: Option<i32>,
    pub max_players: Option<i32>,
}

impl From<Lobby> for LobbyResponse {
    fn from(l: Lobby) -> Self {
        Self {
            id: l.id,
            host_user_id: l.host_user_id,
            host_peer_session_id: l.host_peer_session_id,
            script_url: l.script_url,
            allow_guests: l.allow_guests,
            status: l.status,
            player_count: l.player_count,
            min_players: l.min_players,
            max_players: l.max_players,
        }
    }
}

impl From<LobbyPage> for ListLobbiesResponse {
    fn from(p: LobbyPage) -> Self {
        Self {
            items: p.items.into_iter().map(Into::into).collect(),
            total: p.total,
            page: p.page,
            limit: p.limit,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchLobbyPayload {
    pub allow_guests: Option<bool>,
    pub status: Option<LobbyStatus>,
    pub player_count: Option<u32>,
    pub min_players: Option<Option<u32>>,
    pub max_players: Option<Option<u32>>,
    pub host_peer_session_id: Option<Option<String>>,
    pub script_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalPayload {
    pub to_user_id: Uuid,
    pub signal: serde_json::Value,
}

// ── Validation ───────────────────────────────────────────────────────────────

fn validate_script_url(url: &str) -> Result<(), AppError> {
    let is_github = url.starts_with("https://github.com/")
        || url.starts_with("https://raw.githubusercontent.com/")
        || url.starts_with("https://gist.github.com/")
        || url.starts_with("https://gist.githubusercontent.com/");

    if !is_github {
        return Err(AppError::InvalidScriptUrl(
            "Must be a GitHub, GitHub Raw, or Gist URL".to_string(),
        ));
    }

    if !url.ends_with(".rhai") {
        return Err(AppError::InvalidScriptUrl(
            "Must be a .rhai script".to_string(),
        ));
    }

    // Check for a 40-character hex commit hash
    let commit_hash_regex = regex::Regex::new(r"[0-9a-f]{40}").unwrap();
    if !commit_hash_regex.is_match(url) {
        return Err(AppError::InvalidScriptUrl(
            "Must include a 40-character commit hash for immutability".to_string(),
        ));
    }

    Ok(())
}

// ── SSE event data ────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtSignal {
    lobby_id: Option<Uuid>,
    from_user_id: Uuid,
    signal: serde_json::Value,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn list_lobbies(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListLobbiesQuery>,
) -> Result<impl IntoResponse, AppError> {
    let status = match query.status.as_deref() {
        None => Ok(None),
        Some(s) => LobbyStatus::from_str(s)
            .map(Some)
            .ok_or(AppError::InvalidLobbyStatus),
    }?;

    let params = LobbyListParams {
        page: query.page,
        limit: query.limit,
        allow_guests: query.allow_guests,
        status,
        script_url: query.script_url,
    };
    let page = Lobby::list(&state.db, params).await?;
    Ok(Json(ListLobbiesResponse::from(page)))
}

async fn create_lobby(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Json(payload): Json<CreateLobbyPayload>,
) -> Result<impl IntoResponse, AppError> {
    validate_script_url(&payload.script_url)?;

    let lobby = Lobby::create(
        &state.db,
        NewLobby {
            host_user_id: auth.user_id,
            script_url: payload.script_url,
            allow_guests: payload.allow_guests,
            host_peer_session_id: Some(payload.host_peer_session_id),
            min_players: Some(payload.min_players),
            max_players: Some(payload.max_players),
        },
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreateLobbyResponse { lobby_id: lobby.id }),
    ))
}

async fn get_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let lobby = Lobby::get(&state.db, lobby_id)
        .await?
        .ok_or(AppError::LobbyNotFound)?;
    Ok(Json(LobbyResponse::from(lobby)))
}

async fn patch_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<PatchLobbyPayload>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(ref url) = payload.script_url {
        validate_script_url(url)?;
    }

    let patch = LobbyPatch {
        allow_guests: payload.allow_guests,
        status: payload.status,
        player_count: payload.player_count,
        min_players: payload.min_players,
        max_players: payload.max_players,
        host_peer_session_id: payload.host_peer_session_id,
        script_url: payload.script_url,
    };
    match Lobby::patch(&state.db, lobby_id, auth.user_id, patch).await? {
        None => Err(AppError::LobbyNotFound),
        Some(false) => Err(AppError::Unauthorized),
        Some(true) => Ok(StatusCode::NO_CONTENT),
    }
}

async fn delete_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
) -> Result<impl IntoResponse, AppError> {
    match Lobby::delete(&state.db, lobby_id, auth.user_id).await? {
        None => Err(AppError::LobbyNotFound),
        Some(false) => Err(AppError::Unauthorized),
        Some(true) => Ok(StatusCode::NO_CONTENT),
    }
}

async fn heartbeat(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
) -> Result<impl IntoResponse, AppError> {
    if !Lobby::heartbeat(&state.db, lobby_id, auth.user_id).await? {
        return Err(AppError::LobbyNotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Relay a WebRTC signal to another user within a lobby context.
async fn relay_signal_in_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<SignalPayload>,
) -> Result<impl IntoResponse, AppError> {
    Lobby::get(&state.db, lobby_id)
        .await?
        .ok_or(AppError::LobbyNotFound)?;
    let evt = EvtSignal {
        lobby_id: Some(lobby_id),
        from_user_id: auth.user_id,
        signal: payload.signal,
    };
    state
        .sse_registry
        .send_to(&payload.to_user_id, "signal", &evt)
        .await;
    Ok(StatusCode::NO_CONTENT)
}
