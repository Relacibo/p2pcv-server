# API Reference

Alle Endpoints sind relativ zur API-Base-URL. Authentifizierung erfolgt via JWT Bearer Token im `Authorization: Bearer <token>` Header.

---

# Shared Types

```typescript
type Uuid = string                  // e.g. "f47ac10b-58cc-4372-a567-0e02b2c3d479"
type Timestamp = string             // ISO 8601, e.g. "2024-03-15T09:12:34Z"
type Sha256 = string                // 64-char hex string

type PublicUser = {
  id: Uuid
  userName: string
  avatarHash: Sha256 | null         // null wenn kein Gravatar konfiguriert
  createdAt: Timestamp
}
```

---

# Users

## List Users

```
GET /users
```

Öffentlich.

**Query Parameters**

| Parameter | Typ    | Beschreibung                                                                             |
|-----------|--------|------------------------------------------------------------------------------------------|
| `q`       | string | Filtert nach Nutzernamen (Prefix-Match, case-insensitive)                                |
| `ids`     | string | Kommagetrennte UUIDs – gibt genau diese Nutzer zurück. Ungültige UUIDs werden ignoriert. |

**Response `200`** – `PublicUser[]`

```json
[
  {
    "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "userName": "alice",
    "avatarHash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576e4d8e1456b62c27d",
    "createdAt": "2024-03-15T09:12:34Z"
  },
  {
    "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
    "userName": "bob",
    "avatarHash": null,
    "createdAt": "2024-04-01T18:00:00Z"
  }
]
```

---

## Get User

```
GET /users/:id
```

Authentifizierung erforderlich. Nur der eigene Account.

**Response `200`** – vollständiges User-Objekt  
**Response `401`** – nicht autorisiert

---

## Update User

```
PUT /users/:id
PUT /users/me
```

Authentifizierung erforderlich.

**Schema**

```typescript
type UpdateUserPayload = {
  useGravatar: boolean
  customGravatarEmail?: string      // Leer-String setzt den Custom-Hash zurück
}
```

**Example**

```json
{
  "useGravatar": true,
  "customGravatarEmail": "other@example.com"
}
```

**Response `200`** – aktualisiertes User-Objekt

---

## Delete User

```
DELETE /users/:id
```

Authentifizierung erforderlich. Löscht den eigenen Account inkl. aller verknüpften Daten.

**Response `200`**  
**Response `401`** – nicht autorisiert

---

## List Friends

```
GET /users/:userId/friends
```

Authentifizierung erforderlich.

**Response `200`**

```typescript
type ListFriendsResponse = {
  friends: Array<{
    createdAt: Timestamp
    friend: PublicUser
  }>
}
```

```json
{
  "friends": [
    {
      "createdAt": "2024-06-20T11:45:00Z",
      "friend": {
        "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "userName": "bob",
        "avatarHash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576e4d8e1456b62c27d",
        "createdAt": "2024-04-01T18:00:00Z"
      }
    }
  ]
}
```

---

## Remove Friend

```
DELETE /users/:userId/friends/:friendUserId
```

Authentifizierung erforderlich.

**Response `200`**

---

## Friend Requests – Incoming

```
GET /users/:userId/friend-requests/incoming
```

Authentifizierung erforderlich.

**Response `200`**

```typescript
type IncomingFriendRequestsResponse = {
  receiverId: Uuid
  friendRequests: Array<{
    message: string | null
    createdAt: Timestamp
    sender: PublicUser
  }>
}
```

```json
{
  "receiverId": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "friendRequests": [
    {
      "message": "Hey, lass uns spielen!",
      "createdAt": "2024-07-10T08:30:00Z",
      "sender": {
        "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
        "userName": "bob",
        "avatarHash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576e4d8e1456b62c27d",
        "createdAt": "2024-04-01T18:00:00Z"
      }
    }
  ]
}
```

---

## Friend Requests – Outgoing

```
GET /users/:userId/friend-requests/outgoing
```

Authentifizierung erforderlich.

**Response `200`**

```typescript
type OutgoingFriendRequestsResponse = {
  senderId: Uuid
  friendRequests: Array<{
    message: string | null
    createdAt: Timestamp
    receiver: PublicUser
  }>
}
```

```json
{
  "senderId": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "friendRequests": [
    {
      "message": "Kenn ich dich von Discord?",
      "createdAt": "2024-07-11T14:22:00Z",
      "receiver": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "userName": "carol",
        "avatarHash": "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3",
        "createdAt": "2024-05-05T07:00:00Z"
      }
    }
  ]
}
```

---

## Send Friend Request

```
POST /users/:userId/friend-requests/send-to/:receiverId
```

Authentifizierung erforderlich.

**Schema**

```typescript
type SendFriendRequestPayload = {
  message?: string
}
```

**Example**

```json
{ "message": "Hey, lass uns spielen!" }
```

**Response `200`**  
**Response `400`** – bereits befreundet, oder Anfrage existiert in Gegenrichtung

---

## Delete Friend Request

```
DELETE /users/:userId/friend-requests/by-sender/:senderId
DELETE /users/:userId/friend-requests/by-receiver/:receiverId
```

Authentifizierung erforderlich.

**Response `200`**

---

## Accept Friend Request

```
POST /users/:userId/friend-requests/by-sender/:senderId/accept
```

Authentifizierung erforderlich.

**Response `200`**  
**Response `400`** – Anfrage existiert nicht

---

# Lobby

## Shared Types

```typescript
type LobbyStatus = "waiting" | "in-game" | "finished"

type Lobby = {
  id: Uuid
  hostUserId: Uuid
  hostPeerSessionId: string | null
  scriptUrl: string
  allowGuests: boolean
  status: LobbyStatus
  playerCount: number
  minPlayers: number | null
  maxPlayers: number | null
}
```

---

## List Lobbies

```
GET /lobby
```

Öffentlich. Paginierte Liste aktiver Lobbies.

**Query Parameters**

| Parameter     | Typ     | Default | Beschreibung                                          |
|---------------|---------|---------|-------------------------------------------------------|
| `page`        | integer | `0`     | Seitennummer (0-basiert)                              |
| `limit`       | integer | `20`    | Einträge pro Seite (max. 100)                         |
| `allowGuests` | boolean | –       | Filtert nach `allowGuests`                            |
| `status`      | string  | –       | `"waiting"` \| `"in-game"` \| `"finished"`           |
| `scriptUrl`   | string  | –       | Filtert nach exakter Script-URL                       |

**Response `200`**

```typescript
type ListLobbiesResponse = {
  items: Lobby[]
  total: number
  page: number
  limit: number
}
```

```json
{
  "items": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "hostUserId": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "hostPeerSessionId": "peer-abc123",
      "scriptUrl": "https://github.com/example/game/blob/main/game.js",
      "allowGuests": true,
      "status": "waiting",
      "playerCount": 1,
      "minPlayers": 2,
      "maxPlayers": 4
    }
  ],
  "total": 7,
  "page": 0,
  "limit": 20
}
```

---

## Create Lobby

```
POST /lobby
```

Authentifizierung erforderlich.

**Schema**

```typescript
type CreateLobbyPayload = {
  scriptUrl: string
  allowGuests: boolean              // default: false
}
```

**Example**

```json
{
  "scriptUrl": "https://github.com/example/game/blob/main/game.js",
  "allowGuests": true
}
```

**Response `201`**

```json
{ "lobbyId": "550e8400-e29b-41d4-a716-446655440000" }
```

---

## Get Lobby

```
GET /lobby/:id
```

Öffentlich.

**Response `200`** – `Lobby`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "hostUserId": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "hostPeerSessionId": "peer-abc123",
  "scriptUrl": "https://github.com/example/game/blob/main/game.js",
  "allowGuests": true,
  "status": "waiting",
  "playerCount": 1,
  "minPlayers": 2,
  "maxPlayers": 4
}
```

**Response `404`** – nicht gefunden

---

## Patch Lobby

```
PATCH /lobby/:id
```

Authentifizierung erforderlich. Nur der Host. Alle Felder optional.

**Schema**

```typescript
type PatchLobbyPayload = {
  allowGuests?: boolean
  status?: LobbyStatus
  playerCount?: number
  minPlayers?: number | null
  maxPlayers?: number | null
}
```

**Example**

```json
{
  "status": "in-game",
  "playerCount": 2,
  "minPlayers": 2,
  "maxPlayers": 4
}
```

**Response `204`**  
**Response `401`** – nicht der Host  
**Response `404`** – nicht gefunden

---

## Delete Lobby

```
DELETE /lobby/:id
```

Authentifizierung erforderlich. Nur der Host.

**Response `204`**  
**Response `401`** – nicht der Host  
**Response `404`** – nicht gefunden

---

## Heartbeat

```
POST /lobby/:id/heartbeat
```

Authentifizierung erforderlich. Muss vom Host regelmäßig gesendet werden. Lobbies ohne Heartbeat innerhalb von **300 Sekunden** werden automatisch gelöscht.

**Response `204`**  
**Response `404`** – nicht gefunden oder nicht Host

---

## Signal Relay

```
POST /lobby/:id/signal
```

Authentifizierung erforderlich. Leitet ein WebRTC-Signal via SSE an einen anderen User weiter.

**Schema**

```typescript
type SignalPayload = {
  toUserId: Uuid
  signal: object
}
```

**Example**

```json
{
  "toUserId": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "signal": { "type": "offer", "sdp": "v=0..." }
}
```

**Response `204`**  
**Response `404`** – Lobby nicht gefunden
