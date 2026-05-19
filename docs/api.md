# API Reference

All endpoints are relative to the API base URL. Authentication uses a JWT Bearer token in the `Authorization: Bearer <token>` header.

---

# Shared Types

```typescript
type Uuid = string                  // e.g. "f47ac10b-58cc-4372-a567-0e02b2c3d479"
type Timestamp = string             // ISO 8601, e.g. "2024-03-15T09:12:34Z"
type Sha256 = string                // 64-char hex string

type PublicUser = {
  id: Uuid
  userName: string
  avatarHash?: Sha256
  createdAt: Timestamp
}
```

---

# Users

## List Users

```
GET /users
```

Public.

**Query Parameters**

| Parameter | Type    | Default | Description                                                                       |
|-----------|---------|---------|-----------------------------------------------------------------------------------|
| `page`    | integer | `0`     | Page number (0-based)                                                             |
| `limit`   | integer | `20`    | Items per page (max. 100)                                                         |
| `q`       | string  | –       | Filter by username (prefix match, case-insensitive)                               |
| `ids`     | string  | –       | Comma-separated UUIDs – returns exactly these users. Invalid UUIDs are ignored.   |

**Example Requests**

```
GET /users?q=ali
GET /users?ids=f47ac10b-58cc-4372-a567-0e02b2c3d479,6ba7b810-9dad-11d1-80b4-00c04fd430c8
```

**Response `200`**

```typescript
type ListUsersResponse = {
  items: PublicUser[]
  total: number
  page: number
  limit: number
}
```

```json
{
  "items": [
    {
      "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      "userName": "alice",
      "avatarHash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576e4d8e1456b62c27d",
      "createdAt": "2024-03-15T09:12:34Z"
    },
    {
      "id": "6ba7b810-9dad-11d1-80b4-00c04fd430c8",
      "userName": "bob",
      "createdAt": "2024-04-01T18:00:00Z"
    }
  ],
  "total": 2,
  "page": 0,
  "limit": 20
}
```

---

## Get User

```
GET /users/:id
```

Authentication required. Own account only.

**Schema**

```typescript
type FullUser = {
  id: Uuid
  userName: string
  displayName: string
  email: string
  locale: string
  verifiedEmail: boolean
  createdAt: Timestamp
  updatedAt: Timestamp
  useGravatar: boolean
  customAvatarHash?: Sha256
}
```

**Response `200`**

```json
{
  "id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "userName": "alice",
  "displayName": "Alice",
  "email": "alice@example.com",
  "locale": "en",
  "verifiedEmail": true,
  "createdAt": "2024-03-15T09:12:34Z",
  "updatedAt": "2024-06-01T14:00:00Z",
  "useGravatar": true,
  "customAvatarHash": "b94d27b9934d3e08a52e52d7da7dabfac484efe04294e576e4d8e1456b62c27d"
}
```

**Response `401`** – unauthorized

---

## Patch User

```
PATCH /users/:id
PATCH /users/me
```

Authentication required. All fields are optional; only the provided fields are updated.

**Schema**

```typescript
type PatchUserPayload = {
  useGravatar?: boolean
  customGravatarEmail?: string | null   // null or empty string clears the custom hash
}
```

**Example**

```json
{
  "useGravatar": true,
  "customGravatarEmail": "other@example.com"
}
```

**Response `200`** – updated `FullUser` (same schema as `GET /users/:id`)

---

## Delete User

```
DELETE /users/:id
```

Authentication required. Deletes the own account including all associated data.

**Response `200`**  
**Response `401`** – unauthorized

---

## List Friends

```
GET /users/:userId/friends
```

Authentication required.

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

Authentication required.

**Response `200`**

---

## Friend Requests – Incoming

```
GET /users/:userId/friend-requests/incoming
```

Authentication required.

**Response `200`**

```typescript
type IncomingFriendRequestsResponse = {
  receiverId: Uuid
  friendRequests: Array<{
    message?: string
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
      "message": "Hey, let's play!",
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

Authentication required.

**Response `200`**

```typescript
type OutgoingFriendRequestsResponse = {
  senderId: Uuid
  friendRequests: Array<{
    message?: string
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
      "message": "Do I know you from Discord?",
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

Authentication required.

**Schema**

```typescript
type SendFriendRequestPayload = {
  message?: string
}
```

**Example**

```json
{ "message": "Hey, let's play!" }
```

**Response `200`**  
**Response `400`** – already friends, or a request already exists in the opposite direction

---

## Delete Friend Request

```
DELETE /users/:userId/friend-requests/by-sender/:senderId
DELETE /users/:userId/friend-requests/by-receiver/:receiverId
```

Authentication required.

**Response `200`**

---

## Accept Friend Request

```
POST /users/:userId/friend-requests/by-sender/:senderId/accept
```

Authentication required.

**Response `200`**  
**Response `400`** – request does not exist

---

# Lobby

## Shared Types

```typescript
type LobbyStatus = "waiting" | "in-game" | "finished"

type Lobby = {
  id: Uuid
  hostUserId: Uuid
  hostPeerSessionId?: string
  scriptUrl: string
  allowGuests: boolean
  status: LobbyStatus
  playerCount: number
  minPlayers?: number
  maxPlayers?: number
}
```

---

## List Lobbies

```
GET /lobby
```

Public. Paginated list of active lobbies.

**Query Parameters**

| Parameter     | Type    | Default | Description                                      |
|---------------|---------|---------|--------------------------------------------------|
| `page`        | integer | `0`     | Page number (0-based)                            |
| `limit`       | integer | `20`    | Items per page (max. 100)                        |
| `allowGuests` | boolean | –       | Filter by `allowGuests`                          |
| `status`      | string  | –       | `"waiting"` \| `"in-game"` \| `"finished"`      |
| `scriptUrl`   | string  | –       | Filter by exact script URL                       |

**Example Requests**

```
GET /lobby?status=waiting&allowGuests=true
GET /lobby?scriptUrl=https%3A%2F%2Fgithub.com%2Fexample%2Fgame%2Fblob%2F89e9a545e87c45e183856e01be442c12%2Fgame.rhai&page=1&limit=10
```

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
      "scriptUrl": "https://github.com/example/game/blob/89e9a545e87c45e183856e01be442c12/game.rhai",
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

Authentication required.

**Schema**

```typescript
type CreateLobbyPayload = {
  scriptUrl: string                 // Must be GitHub/Gist, end in .rhai, and include a commit hash
  allowGuests: boolean
  hostPeerSessionId: string
  minPlayers: number
  maxPlayers: number
}
```

**Example**

```json
{
  "scriptUrl": "https://github.com/example/game/blob/89e9a545e87c45e183856e01be442c12/game.rhai",
  "allowGuests": true,
  "hostPeerSessionId": "peer-abc123",
  "minPlayers": 2,
  "maxPlayers": 4
}
```

**Response `201`**

```json
{ "lobbyId": "550e8400-e29b-41d4-a716-446655440000" }
```

**Response `400`** – invalid script URL (e.g., missing commit hash, not GitHub, not .rhai)

---

## Get Lobby

```
GET /lobby/:id
```

Public.

**Response `200`** – `Lobby`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "hostUserId": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "hostPeerSessionId": "peer-abc123",
  "scriptUrl": "https://github.com/example/game/blob/89e9a545e87c45e183856e01be442c12/game.rhai",
  "allowGuests": true,
  "status": "waiting",
  "playerCount": 1,
  "minPlayers": 2,
  "maxPlayers": 4
}
```

**Response `404`** – not found

---

## Patch Lobby

```
PATCH /lobby/:id
```

Authentication required. Host only. All fields optional.

**Schema**

```typescript
type PatchLobbyPayload = {
  allowGuests?: boolean
  status?: LobbyStatus
  playerCount?: number
  minPlayers?: number | null
  maxPlayers?: number | null
  hostPeerSessionId?: string | null
  scriptUrl?: string                // Same validation rules as Create Lobby
}
```

**Example**

```json
{
  "status": "in-game",
  "playerCount": 2,
  "hostPeerSessionId": "peer-xyz789",
  "scriptUrl": "https://github.com/example/game/blob/f089b1f00a0c07a7e956d20f18ac35ffe34c1c86/game.rhai"
}
```

**Response `204`**  
**Response `401`** – not the host  
**Response `404`** – not found

---

## Delete Lobby

```
DELETE /lobby/:id
```

Authentication required. Host only.

**Response `204`**  
**Response `401`** – not the host  
**Response `404`** – not found

---

## Heartbeat

```
POST /lobby/:id/heartbeat
```

Authentication required. Must be sent by the host regularly. Lobbies without a heartbeat within **300 seconds** are automatically deleted.

**Response `204`**  
**Response `404`** – not found or not host

---

## Signal Relay

```
POST /lobby/:id/signal
```

Authentication required. Forwards a WebRTC signal via SSE to another user.

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
**Response `404`** – lobby not found


---

# WebRTC

## Get TURN Credentials

```
GET /turn-credentials
```

Authentication required. Returns short-lived TURN server credentials for WebRTC peer connections, using coturn's [HMAC secret mechanism](https://github.com/coturn/coturn/wiki/turnserver#turn-rest-api).

**Response `200`**

```typescript
type TurnCredentials = {
  urls: string[]        // TURN server URIs, e.g. "turn:example.com:3478"
  username: string      // "{expiry_unix_timestamp}:{userId}"
  credential: string    // base64(HMAC-SHA1(secret, username))
  ttl: number           // Lifetime in seconds (86400 = 24 h)
}
```

```json
{
  "urls": ["turn:ovilava.rcbnetwork.de:3478"],
  "username": "1747609200:f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "credential": "Wr0opBf2N3q9G7MkJ2X4Lz5eHKA=",
  "ttl": 86400
}
```

**Response `401`** – not authenticated  
**Response `503`** – TURN server not configured on this instance
