use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppState, api::auth::session::auth::Auth, error::AppError};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_lobby))
        .route("/{id}", get(get_lobby))
        .route("/{id}", delete(delete_lobby))
        .route("/{id}/heartbeat", post(heartbeat))
        .route("/{id}/host", post(update_host))
        .route("/{id}/settings", post(update_settings))
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
    pub script_url: String,
    pub allow_guests: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateHostPayload {
    pub new_host_user_id: Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsPayload {
    pub allow_guests: bool,
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
        script_url: lobby.script_url,
        allow_guests: lobby.allow_guests,
    }))
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

/// Called by the new host after detecting the previous host disconnected.
async fn update_host(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<UpdateHostPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Only the declared new host can call this
    if payload.new_host_user_id != auth.user_id {
        return Err(AppError::Unauthorized);
    }
    if !state
        .lobby_registry
        .update_host(&lobby_id, auth.user_id)
    {
        return Err(AppError::LobbyNotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn update_settings(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<UpdateSettingsPayload>,
) -> Result<impl IntoResponse, AppError> {
    let lobby = state
        .lobby_registry
        .get(&lobby_id)
        .ok_or(AppError::LobbyNotFound)?;
    if lobby.host_user_id != auth.user_id {
        return Err(AppError::Unauthorized);
    }
    state.lobby_registry.update_settings(&lobby_id, payload.allow_guests);
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
