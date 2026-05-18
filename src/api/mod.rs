use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{AppState, api::auth::session::auth::Auth, error::AppError};

pub mod auth;
pub mod events;
pub mod games;
pub mod lobby;
pub mod users;

pub fn router() -> Router<Arc<crate::AppState>> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/games", games::router())
        .nest("/lobby", lobby::router())
        .route("/events", get(events::sse_handler))
        .route("/signal/{target_user_id}", post(relay_signal_direct))
}

// ── Generic signal relay (no lobby context) ───────────────────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DirectSignalPayload {
    signal: serde_json::Value,
}

#[derive(serde::Serialize)]
#[serde_with::skip_serializing_none]
#[serde(rename_all = "camelCase")]
struct EvtDirectSignal {
    lobby_id: Option<Uuid>,
    from_user_id: Uuid,
    signal: serde_json::Value,
}

async fn relay_signal_direct(
    State(state): State<Arc<AppState>>,
    Path(target_user_id): Path<Uuid>,
    auth: Auth,
    Json(payload): Json<DirectSignalPayload>,
) -> Result<impl IntoResponse, AppError> {
    let evt = EvtDirectSignal {
        lobby_id: None,
        from_user_id: auth.user_id,
        signal: payload.signal,
    };
    state
        .sse_registry
        .send_to(&target_user_id, "signal", &evt)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

