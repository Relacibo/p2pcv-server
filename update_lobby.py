import re
with open("src/lobby.rs", "r") as f:
    data = f.read()

# Add allow_guests to Lobby
data = data.replace('pub script_url: String,\n    pub created_at: Instant,', 'pub script_url: String,\n    pub allow_guests: bool,\n    pub created_at: Instant,')

# Update Lobby::new
data = data.replace('pub fn new(host_user_id: Uuid, script_url: String) -> Self {', 'pub fn new(host_user_id: Uuid, script_url: String, allow_guests: bool) -> Self {')
data = data.replace('script_url,\n            created_at: now,', 'script_url,\n            allow_guests,\n            created_at: now,')

# Update LobbyRegistry::create
data = data.replace('pub fn create(&self, host_user_id: Uuid, script_url: String) -> Lobby {', 'pub fn create(&self, host_user_id: Uuid, script_url: String, allow_guests: bool) -> Lobby {')
data = data.replace('let lobby = Lobby::new(host_user_id, script_url);', 'let lobby = Lobby::new(host_user_id, script_url, allow_guests);')

# Add update_settings
update_fn = """    pub fn update_settings(&self, lobby_id: &Uuid, allow_guests: bool) -> bool {
        if let Some(mut entry) = self.0.get_mut(lobby_id) {
            entry.allow_guests = allow_guests;
            return true;
        }
        false
    }

    /// Remove lobbies"""
data = data.replace('/// Remove lobbies', update_fn)

with open("src/lobby.rs", "w") as f:
    f.write(data)
