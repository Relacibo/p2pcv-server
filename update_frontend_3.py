import re
with open("/home/reinhard/git/p2p-chessvariants/src/api/lobbyApi.ts", "r") as f:
    data = f.read()

# lobbyApi.createLobby signature
data = data.replace('export async function createLobby(\n  scriptUrl: string,\n  token: string\n): Promise<{ lobbyId: string }> {', 'export async function createLobby(\n  scriptUrl: string,\n  allowGuests: boolean,\n  token: string\n): Promise<{ lobbyId: string }> {')
data = data.replace('body: JSON.stringify({ scriptUrl }),', 'body: JSON.stringify({ scriptUrl, allowGuests }),')

with open("/home/reinhard/git/p2p-chessvariants/src/api/lobbyApi.ts", "w") as f:
    f.write(data)
