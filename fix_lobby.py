import re
with open("/home/reinhard/git/p2pcv-server/src/api/lobby/mod.rs", "r") as f:
    data = f.read()

# Add security check to relay_signal_in_lobby
sec_check = """
    let _lobby = state
        .lobby_registry
        .get(&lobby_id)
        .ok_or(AppError::LobbyNotFound)?;
        
    if !_lobby.allow_guests && auth.is_guest {
        return Err(AppError::Unauthorized);
    }
"""

data = re.sub(r'    let _lobby = state\n        \.lobby_registry\n        \.get\(&lobby_id\)\n        \.ok_or\(AppError::LobbyNotFound\)\?;', sec_check.strip(), data, flags=re.DOTALL)

with open("/home/reinhard/git/p2pcv-server/src/api/lobby/mod.rs", "w") as f:
    f.write(data)
