import re
with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/lobbySlice.ts", "r") as f:
    data = f.read()

# createLobby signature
data = data.replace('export function createLobby(scriptUrl: string, useServerLobby: boolean = false): AppThunk<Promise<void>> {', 'export function createLobby(scriptUrl: string, useServerLobby: boolean = false, allowGuests: boolean = true): AppThunk<Promise<void>> {')

# lobbyApi.createLobby call
data = data.replace('const res = await lobbyApi.createLobby(scriptUrl, token);', 'const res = await lobbyApi.createLobby(scriptUrl, allowGuests, token);')

with open("/home/reinhard/git/p2p-chessvariants/src/features/lobby/lobbySlice.ts", "w") as f:
    f.write(data)
