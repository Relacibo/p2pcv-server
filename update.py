import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/lobbySlice.ts", "r") as f:
    data = f.read()

data = data.replace('export function createLobby(scriptUrl: string): AppThunk<Promise<void>> {', 'export function createLobby(scriptUrl: string, useServerLobby: boolean = false): AppThunk<Promise<void>> {')

if 'buildPeerInviteFragment' not in data:
    data = data.replace('buildLobbyInviteFragment } from "./scriptUrl";', 'buildLobbyInviteFragment, buildPeerInviteFragment } from "./scriptUrl";')

old_try = """    try {
      const { lobbyId } = await lobbyApi.createLobby(scriptUrl, token);
      dispatch(_setLocalUserId(user.id));
      dispatch(_setServerLobbyId(lobbyId));
      dispatch(_playerJoined({ userId: user.id, name: user.displayName ?? null, ready: false }));

      // Init P2P as host
      webrtcService.init((toUserId, signal) =>
        lobbyApi.sendSignal(lobbyId, toUserId, signal, token)
      );
      p2pLobbyService.initP2PLobby(user.id, true, lobbyId, token, {
        onLobbyInfo: () => {},
        onPlayerJoined: (player) =>
          dispatch(_playerJoined({ userId: player.userId, name: player.displayName, ready: false })),
        onPlayerLeft: (userId) => dispatch(_playerLeft(userId)),
        onHostMigration: (_newHost) => {},
        onGameMessage: () => {},
      });

      const inviteUrl =
        window.location.origin + "/lobby#" + buildLobbyInviteFragment(lobbyId);
      dispatch(_setHosting(inviteUrl));
    }"""

new_try = """    try {
      let lobbyId: string | null = null;
      if (useServerLobby && token) {
        const res = await lobbyApi.createLobby(scriptUrl, token);
        lobbyId = res.lobbyId;
        dispatch(_setServerLobbyId(lobbyId));
      }

      dispatch(_setLocalUserId(user.id));
      dispatch(_playerJoined({ userId: user.id, name: user.displayName ?? null, ready: false }));

      // Init P2P as host
      if (useServerLobby && token && lobbyId) {
        webrtcService.init((toUserId, signal) =>
          lobbyApi.sendSignal(lobbyId as string, toUserId, signal, token)
        );
      } else {
        webrtcService.init((toUserId, signal) =>
          lobbyApi.sendSignalDirect(toUserId, signal, token || "")
        );
      }

      p2pLobbyService.initP2PLobby(user.id, true, lobbyId, token || "", {
        onLobbyInfo: () => {},
        onPlayerJoined: (player) =>
          dispatch(_playerJoined({ userId: player.userId, name: player.displayName, ready: false })),
        onPlayerLeft: (userId) => dispatch(_playerLeft(userId)),
        onHostMigration: (_newHost) => {},
        onGameMessage: () => {},
      });

      const inviteUrl = useServerLobby && lobbyId
        ? window.location.origin + "/lobby#" + buildLobbyInviteFragment(lobbyId)
        : window.location.origin + "/lobby#" + buildPeerInviteFragment(user.id);
        
      dispatch(_setHosting(inviteUrl));
    }"""

data = data.replace(old_try, new_try)

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/lobbySlice.ts", "w") as f:
    f.write(data)
