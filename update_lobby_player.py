import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/ActiveLobbyView.tsx", "r") as f:
    data = f.read()

# Update player list to show guest badge
# Assuming players with "Guest" in their name or players without registered status
# The backend doesn't explicitly flag them except by username starting with "Guest " and having `verified_email` false. But `LobbyPlayer` doesn't have that.
# Let's check `LobbyPlayer` in lobbySlice: { userId: string, name: string | null, ready: boolean }
# A simple heuristic: if name starts with "Guest " it's a guest.

badge_code = """<Text>{p.name || "Anonymous"}</Text>
                    {p.name?.startsWith("Guest ") && (
                      <Badge color="gray" size="sm" variant="outline" ml="xs">Guest</Badge>
                    )}"""
data = data.replace('<Text>{p.name || "Anonymous"}</Text>', badge_code)

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/ActiveLobbyView.tsx", "w") as f:
    f.write(data)
