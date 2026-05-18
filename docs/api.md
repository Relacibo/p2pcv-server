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

## Update User

```
PUT /users/:id
PUT /users/me
```

Authentication required.

**Schema**

```typescript
type UpdateUserPayload = {
  useGravatar: boolean
  customGravatarEmail?: string      // empty string resets the custom hash
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
GET /lobby?scriptUrl=https%3A%2F%2Fgithub.com%2Fexample%2Fgame%2Fblob%2Fmain%2Fgame.js&page=1&limit=10
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

Authentication required.

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

Public.

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
}
```

**Example**

```json
{
  "status": "in-game",
  "playerCount": 2
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
