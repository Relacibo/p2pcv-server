use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, api::auth::session::auth::Auth, error::AppError, lobby::{LobbyPatch, LobbyStatus}};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
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
pub struct CreateLobbyPayload {
    pub script_url: String,
    #[serde(default)]
    pub allow_guests: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyResponse {
    pub lobby_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyResponse {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub host_peer_session_id: Option<String>,
    pub script_url: String,
    pub allow_guests: bool,
    pub status: LobbyStatus,
    pub player_count: u32,
    pub min_players: Option<u32>,
    pub max_players: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchLobbyPayload {
    pub allow_guests: Option<bool>,
    pub status: Option<LobbyStatus>,
    pub player_count: Option<u32>,
    pub min_players: Option<Option<u32>>,
    pub max_players: Option<Option<u32>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalPayload {
    pub to_user_id: Uuid,
    pub signal: serde_json::Value,
}

// ── SSE event data ────────────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtLobbyDeleted {
    lobby_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtSignal {
    lobby_id: Option<Uuid>,
    from_user_id: Uuid,
    signal: serde_json::Value,
}

// ── Handlers ──────────────────────────────────────────────────────────────────

async fn create_lobby(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Json(payload): Json<CreateLobbyPayload>,
) -> Result<impl IntoResponse, AppError> {
    let lobby = state
        .lobby_registry
        .create(auth.user_id, payload.script_url, payload.allow_guests);
    Ok((
        StatusCode::CREATED,
        Json(CreateLobbyResponse { lobby_id: lobby.id }),
    ))
}

async fn get_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let lobby = state
        .lobby_registry
        .get(&lobby_id)
        .ok_or(AppError::LobbyNotFound)?;
    Ok(Json(LobbyResponse {
        id: lobby.id,
        host_user_id: lobby.host_user_id,
        host_peer_session_id: lobby.host_peer_session_id.clone(),
        script_url: lobby.script_url,
        allow_guests: lobby.allow_guests,
        status: lobby.status,
        player_count: lobby.player_count,
        min_players: lobby.min_players,
        max_players: lobby.max_players,
    }))
}

async fn patch_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<PatchLobbyPayload>,
) -> Result<impl IntoResponse, AppError> {
    let patch = LobbyPatch {
        allow_guests: payload.allow_guests,
        status: payload.status,
        player_count: payload.player_count,
        min_players: payload.min_players,
        max_players: payload.max_players,
    };
    match state.lobby_registry.patch(&lobby_id, &auth.user_id, patch) {
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
    let lobby = state
        .lobby_registry
        .get(&lobby_id)
        .ok_or(AppError::LobbyNotFound)?;
    if lobby.host_user_id != auth.user_id {
        return Err(AppError::Unauthorized);
    }
    state.lobby_registry.delete(&lobby_id);
    Ok(StatusCode::NO_CONTENT)
}

async fn heartbeat(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
) -> Result<impl IntoResponse, AppError> {
    if !state.lobby_registry.heartbeat(&lobby_id, &auth.user_id) {
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
    state
        .lobby_registry
        .get(&lobby_id)
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

