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

use crate::{
    AppState,
    api::auth::session::auth::Auth,
    db::users::User,
    error::AppError,
    lobby::{JoinError, LobbyMember, LobbyStatus, StartGameError},
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_lobby))
        .route("/{id}", get(get_lobby))
        .route("/{id}", delete(delete_lobby))
        .route("/{id}/join", post(join_lobby))
        .route("/{id}/leave", post(leave_lobby))
        .route("/{id}/start", post(start_game))
        .route("/{id}/game-ended", post(game_ended))
        .route("/{id}/invite", post(invite_to_lobby))
        .route("/{id}/signal", post(relay_signal))
}

// ── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyPayload {
    pub script_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLobbyResponse {
    pub lobby_id: Uuid,
    pub script_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LobbyResponse {
    pub id: Uuid,
    pub host_user_id: Uuid,
    pub members: Vec<LobbyMember>,
    pub status: LobbyStatus,
    pub script_url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvitePayload {
    pub user_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalPayload {
    pub to_user_id: Uuid,
    pub signal: serde_json::Value,
}

// ── SSE event data types ──────────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtMemberJoined {
    lobby_id: Uuid,
    member: LobbyMember,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtMemberLeft {
    lobby_id: Uuid,
    user_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtLobbyDeleted {
    lobby_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtGameStarted {
    lobby_id: Uuid,
    members: Vec<LobbyMember>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtGameEnded {
    lobby_id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtLobbyInvite {
    lobby_id: Uuid,
    host_user_id: Uuid,
    host_display_name: String,
    script_url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EvtSignal {
    lobby_id: Uuid,
    from_user_id: Uuid,
    signal: serde_json::Value,
}

// ── Handlers ─────────────────────────────────────────────────────────────────

async fn create_lobby(
    State(state): State<Arc<AppState>>,
    auth: Auth,
    Json(payload): Json<CreateLobbyPayload>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::get(&state.db, auth.user_id).await?;
    let lobby = state
        .lobby_registry
        .create(auth.user_id, user.display_name, payload.script_url.clone());
    Ok((
        StatusCode::CREATED,
        Json(CreateLobbyResponse {
            lobby_id: lobby.id,
            script_url: payload.script_url,
        }),
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
        members: lobby.members,
        status: lobby.status,
        script_url: lobby.script_url,
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
    let member_ids = lobby.member_user_ids();
    state.lobby_registry.delete(&lobby_id);
    state
        .sse_registry
        .broadcast_to(&member_ids, "lobby_deleted", &EvtLobbyDeleted { lobby_id })
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn join_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
) -> Result<impl IntoResponse, AppError> {
    let user = User::get(&state.db, auth.user_id).await?;
    let (member, all_member_ids) = state
        .lobby_registry
        .join(&lobby_id, auth.user_id, user.display_name)
        .map_err(|e| match e {
            JoinError::NotFound => AppError::LobbyNotFound,
            JoinError::GameAlreadyStarted => AppError::LobbyGameAlreadyStarted,
        })?;

    // Notify all existing members (except the joiner) that someone joined
    let others: Vec<Uuid> = all_member_ids
        .iter()
        .copied()
        .filter(|id| *id != auth.user_id)
        .collect();
    state
        .sse_registry
        .broadcast_to(
            &others,
            "lobby_member_joined",
            &EvtMemberJoined {
                lobby_id,
                member: member.clone(),
            },
        )
        .await;

    Ok(Json(serde_json::json!({ "lobbyId": lobby_id })))
}

async fn leave_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
) -> Result<impl IntoResponse, AppError> {
    let (remaining_ids, was_host) = state
        .lobby_registry
        .leave(&lobby_id, &auth.user_id)
        .ok_or(AppError::LobbyNotFound)?;

    if was_host || remaining_ids.is_empty() {
        // Host left → delete the whole lobby
        state.lobby_registry.delete(&lobby_id);
        state
            .sse_registry
            .broadcast_to(
                &remaining_ids,
                "lobby_deleted",
                &EvtLobbyDeleted { lobby_id },
            )
            .await;
    } else {
        state
            .sse_registry
            .broadcast_to(
                &remaining_ids,
                "lobby_member_left",
                &EvtMemberLeft {
                    lobby_id,
                    user_id: auth.user_id,
                },
            )
            .await;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn start_game(
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
    let members = state
        .lobby_registry
        .start_game(&lobby_id)
        .map_err(|e| match e {
            StartGameError::NotFound => AppError::LobbyNotFound,
            StartGameError::AlreadyStarted => AppError::LobbyGameAlreadyStarted,
            StartGameError::NotEnoughPlayers => AppError::LobbyNotEnoughPlayers,
        })?;
    let member_ids: Vec<Uuid> = members.iter().map(|m| m.user_id).collect();
    state
        .sse_registry
        .broadcast_to(
            &member_ids,
            "lobby_game_started",
            &EvtGameStarted {
                lobby_id,
                members: members.clone(),
            },
        )
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn game_ended(
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
    let member_ids = state
        .lobby_registry
        .set_game_ended(&lobby_id)
        .ok_or(AppError::LobbyNotFound)?;
    state
        .sse_registry
        .broadcast_to(&member_ids, "lobby_game_ended", &EvtGameEnded { lobby_id })
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn invite_to_lobby(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<InvitePayload>,
) -> Result<impl IntoResponse, AppError> {
    let lobby = state
        .lobby_registry
        .get(&lobby_id)
        .ok_or(AppError::LobbyNotFound)?;
    if !lobby.members.iter().any(|m| m.user_id == auth.user_id) {
        return Err(AppError::Unauthorized);
    }
    let inviter = lobby
        .members
        .iter()
        .find(|m| m.user_id == auth.user_id)
        .unwrap();
    let evt = EvtLobbyInvite {
        lobby_id,
        host_user_id: lobby.host_user_id,
        host_display_name: inviter.display_name.clone(),
        script_url: lobby.script_url.clone(),
    };
    for user_id in &payload.user_ids {
        state.sse_registry.send_to(user_id, "lobby_invite", &evt).await;
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn relay_signal(
    State(state): State<Arc<AppState>>,
    Path(lobby_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<SignalPayload>,
) -> Result<impl IntoResponse, AppError> {
    let lobby = state
        .lobby_registry
        .get(&lobby_id)
        .ok_or(AppError::LobbyNotFound)?;
    if !lobby.members.iter().any(|m| m.user_id == auth.user_id) {
        return Err(AppError::Unauthorized);
    }
    if !lobby.members.iter().any(|m| m.user_id == payload.to_user_id) {
        return Err(AppError::LobbyNotMember);
    }
    let evt = EvtSignal {
        lobby_id,
        from_user_id: auth.user_id,
        signal: payload.signal,
    };
    state
        .sse_registry
        .send_to(&payload.to_user_id, "signal", &evt)
        .await;
    Ok(StatusCode::NO_CONTENT)
}
