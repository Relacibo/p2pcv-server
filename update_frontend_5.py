import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/lobbySlice.ts", "r") as f:
    data = f.read()

# Add notifications for lobby creation
if 'import { notifications }' not in data:
    data = data.replace('import { buildLobbyInviteFragment, buildPeerInviteFragment } from "./scriptUrl";', 'import { buildLobbyInviteFragment, buildPeerInviteFragment } from "./scriptUrl";\nimport { notifications } from "@mantine/notifications";')

data = data.replace('dispatch(_setHosting(inviteUrl));\n    } catch (err) {', 'dispatch(_setHosting(inviteUrl));\n      notifications.show({ title: "Lobby created!", message: "Share the invite link with players.", color: "green" });\n    } catch (err) {')

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/lobbySlice.ts", "w") as f:
    f.write(data)
