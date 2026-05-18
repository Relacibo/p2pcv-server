import re
with open("src/api/lobby/mod.rs", "r") as f:
    data = f.read()

# Add route
data = data.replace('.route("/{id}/host", post(update_host))', '.route("/{id}/host", post(update_host))\n        .route("/{id}/settings", post(update_settings))')

# Update payloads
data = data.replace('pub struct CreateLobbyPayload {\n    pub script_url: String,\n}', 'pub struct CreateLobbyPayload {\n    pub script_url: String,\n    #[serde(default)]\n    pub allow_guests: bool,\n}')
data = data.replace('pub struct LobbyResponse {\n    pub id: Uuid,\n    pub host_user_id: Uuid,\n    pub script_url: String,\n}', 'pub struct LobbyResponse {\n    pub id: Uuid,\n    pub host_user_id: Uuid,\n    pub script_url: String,\n    pub allow_guests: bool,\n}')

payload_add = """#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSettingsPayload {
    pub allow_guests: bool,
}

"""
data = data.replace('pub struct SignalPayload', payload_add + 'pub struct SignalPayload')

# Update handlers
data = data.replace('.create(auth.user_id, payload.script_url);', '.create(auth.user_id, payload.script_url, payload.allow_guests);')
data = data.replace('script_url: lobby.script_url,\n    }))', 'script_url: lobby.script_url,\n        allow_guests: lobby.allow_guests,\n    }))')

update_settings_fn = """async fn update_settings(
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

/// Relay a WebRTC signal"""
data = data.replace('/// Relay a WebRTC signal', update_settings_fn)

with open("src/api/lobby/mod.rs", "w") as f:
    f.write(data)
