use std::{
    collections::HashMap,
    env,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    net::Ipv4Addr,
    str::FromStr,
    time::Duration,
};

use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use dashmap::DashMap;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::{
    core::muxing::StreamMuxerBox,
    gossipsub,
    identity::Keypair,
    multiaddr::Protocol,
    ping,
    request_response::{self, Codec, OutboundRequestId, ProtocolSupport, ResponseChannel},
    swarm::{NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm, SwarmBuilder, Transport, TransportError,
};
use libp2p_webrtc::tokio::Certificate;
use p2pcv_bebop::{C2sMsg, Guid, GuidExt, S2cMsg, UuidExt};
use thiserror::Error;
use uuid::Uuid;

use crate::{
    api::auth::session::{claims::Claims, config::Config as JwtConfig},
    db::{db_conn::DbPool, users::User},
};

pub async fn init(db: DbPool, jwt_config: JwtConfig) -> Result<(), P2pError> {
    let timeout_secs = env::var("P2P_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);
    let host = env::var("P2P_HOST").expect("P2P_HOST needed!");
    let address = Ipv4Addr::from_str(&host).expect("Invalid P2P_HOST address");
    let port: u16 = env::var("P2P_PORT")
        .expect("P2P_PORT needed!")
        .parse()
        .expect("P2P_PORT not a number");
    let private_token = crate::secret::read_secret("P2P_PRIVATE_KEY_ED25519");
    let mut private_key = BASE64_STANDARD
        .decode(private_token)
        .expect("P2P_PRIVATE_KEY_ED25519 is invalid base64");
    // Only the 32 last bytes are the actual key
    private_key = private_key
        .into_iter()
        .rev()
        .take(32)
        .rev()
        .collect::<Vec<_>>();

    let keypair = Keypair::ed25519_from_bytes(private_key)
        .expect("P2P_PRIVATE_KEY_ED25519 is not a private key");

    let cert = Certificate::generate(&mut rand::thread_rng())
        .expect("Failed to generate WebRTC certificate");

    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_other_transport(|id_keys| {
            let transport = libp2p_webrtc::tokio::Transport::new(
                id_keys.clone(),
                cert,
            );
            let res = transport.map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)));
            Ok(res)
        })
        .expect("Could not add WebRTC transport")
        .with_behaviour(Behaviour::create)
        .map_err(|_| P2pError::InitP2p)?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(timeout_secs)))
        .build();

    let address_webrtc = Multiaddr::from(address)
        .with(Protocol::Udp(port))
        .with(Protocol::WebRTCDirect);

    swarm.listen_on(address_webrtc)?;

    let registry = PeerRegistry::default();
    let mut pending: HashMap<OutboundRequestId, PendingNewGame> = HashMap::new();

    loop {
        let event = tokio::select! {
            event = swarm.select_next_some() => event,
            _ = tokio::signal::ctrl_c() => break,
        };
        if let Err(err) =
            handle_swarm_event(&mut swarm, &db, &jwt_config, &registry, &mut pending, event).await
        {
            log::error!("p2p event error: {err}");
        }
    }

    Ok(())
}

async fn handle_swarm_event(
    swarm: &mut Swarm<Behaviour>,
    db: &DbPool,
    jwt_config: &JwtConfig,
    registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    event: SwarmEvent<BehaviourEvent>,
) -> Result<(), P2pError> {
    log::debug!("{:?}", event);
    match event {
        SwarmEvent::Behaviour(BehaviourEvent::C2s(ev)) => {
            handle_c2s_event(swarm, db, jwt_config, registry, pending, ev).await?;
        }
        SwarmEvent::Behaviour(BehaviourEvent::S2c(ev)) => {
            handle_s2c_event(swarm, registry, pending, ev).await?;
        }
        SwarmEvent::NewListenAddr { address, .. } => {
            log::info!("Listening on {address}");
        }
        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
            log::debug!("Connection established: {peer_id}");
        }
        SwarmEvent::ConnectionClosed { peer_id, cause, .. } => {
            log::debug!("Connection closed: {peer_id} ({cause:?})");
            registry.remove_peer(&peer_id);
        }
        _ => {}
    }
    Ok(())
}

async fn handle_c2s_event(
    swarm: &mut Swarm<Behaviour>,
    db: &DbPool,
    jwt_config: &JwtConfig,
    registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    event: request_response::Event<C2sMsg, S2cMsg>,
) -> Result<(), P2pError> {
    use request_response::{Event, Message};
    match event {
        Event::Message {
            peer,
            message: Message::Request {
                request, channel, ..
            },
            ..
        } => {
            let Some(c2s) = Some(request) else {
                log::warn!("Received empty c2s message from {peer}");
                return Ok(());
            };
            match c2s {
                C2sMsg::RegisterPeer { auth_token } => {
                    handle_register_peer(swarm, jwt_config, registry, peer, auth_token, channel)
                        .await;
                }
                C2sMsg::GetFriendPeerId { friend_user_id } => {
                    handle_get_friend_peer_id(swarm, db, registry, peer, friend_user_id, channel)
                        .await;
                }
                C2sMsg::NewGame { receiver_user_id, variant_id, variant_version } => {
                    handle_new_game(
                        swarm,
                        db,
                        registry,
                        pending,
                        peer,
                        receiver_user_id,
                        variant_id,
                        variant_version,
                        channel,
                    )
                    .await;
                }
                C2sMsg::NewGameAnswer { .. } => {
                    log::warn!("Unexpected NewGameAnswer via c2s from {peer}; ignoring");
                }
                C2sMsg::Unknown => {
                    log::warn!("Received unknown C2S message variant from {peer}; ignoring");
                }
            }
        }
        Event::OutboundFailure {
            peer,
            request_id,
            error,
            ..
        } => {
            log::warn!("c2s outbound failure to {peer}: {error:?} (id={request_id:?})");
        }
        Event::InboundFailure {
            peer,
            request_id,
            error,
            ..
        } => {
            log::warn!("c2s inbound failure from {peer}: {error:?} (id={request_id:?})");
        }
        _ => {}
    }
    Ok(())
}

async fn handle_register_peer(
    swarm: &mut Swarm<Behaviour>,
    jwt_config: &JwtConfig,
    registry: &PeerRegistry,
    peer: PeerId,
    auth_token: String,
    channel: ResponseChannel<S2cMsg>,
) {
    let resp = match validate_jwt(jwt_config, &auth_token) {
        Ok(claims) => {
            let user_id = claims.sub;
            registry.register(user_id, peer);
            log::info!("Registered peer {peer} for user {user_id}");
            S2cMsg::RegisterPeerResponse { success: true }
        }
        Err(err) => {
            log::warn!("RegisterPeer auth failed for {peer}: {err}");
            S2cMsg::RegisterPeerResponse { success: false }
        }
    };
    if swarm
        .behaviour_mut()
        .c2s
        .send_response(channel, resp)
        .is_err()
    {
        log::warn!("Could not send RegisterPeerResponse to {peer}: channel closed");
    }
}

async fn handle_get_friend_peer_id(
    swarm: &mut Swarm<Behaviour>,
    db: &DbPool,
    registry: &PeerRegistry,
    peer: PeerId,
    friend_user_id: Guid,
    channel: ResponseChannel<S2cMsg>,
) {
    let peer_id_str = async {
        let requester_user_id = registry.user_id_for_peer(&peer)?;
        let friend_user_id: Uuid = friend_user_id.to_uuid();
        if friend_user_id.is_nil() {
            return None;
        }

        let is_friends = User::is_friends_with(db, requester_user_id, friend_user_id)
            .await
            .ok()?;
        if !is_friends {
            return None;
        }
        let friend_peer = registry.peer_id_for_user(&friend_user_id)?;
        Some(friend_peer.to_string())
    }
    .await;

    let resp = S2cMsg::GetFriendPeerIdResponse { peer_id: peer_id_str };
    if swarm
        .behaviour_mut()
        .c2s
        .send_response(channel, resp)
        .is_err()
    {
        log::warn!("Could not send GetFriendPeerIdResponse to {peer}: channel closed");
    }
}

async fn handle_new_game(
    swarm: &mut Swarm<Behaviour>,
    db: &DbPool,
    registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    peer: PeerId,
    receiver_user_id: Guid,
    variant_id: Guid,
    variant_version: String,
    channel: ResponseChannel<S2cMsg>,
) {
    let result: Result<(PeerId, Uuid, S2cMsg), ()> = async {
        let sender_user_id = registry.user_id_for_peer(&peer).ok_or(())?;

        let receiver_user_id: Uuid = receiver_user_id.to_uuid();
        if receiver_user_id.is_nil() {
            return Err(());
        }

        let is_friends = User::is_friends_with(db, sender_user_id, receiver_user_id)
            .await
            .map_err(|_| ())?;
        if !is_friends {
            return Err(());
        }

        let receiver_peer = registry.peer_id_for_user(&receiver_user_id).ok_or(())?;

        let sender_user = User::get(db, sender_user_id).await.map_err(|_| ())?;

        let event = S2cMsg::NewGameEvent {
            sender_user_id: sender_user_id.to_guid(),
            sender_user_name: sender_user.user_name,
            variant_id,
            variant_version,
            timeout_secs: 60,
        };
        Ok((receiver_peer, sender_user_id, event))
    }
    .await;

    match result {
        Ok((receiver_peer, sender_user_id, event)) => {
            let request_id = swarm.behaviour_mut().s2c.send_request(&receiver_peer, event);
            pending.insert(
                request_id,
                PendingNewGame {
                    channel,
                    sender_user_id,
                    receiver_peer_id: receiver_peer,
                },
            );
            log::debug!("NewGame forwarded to {receiver_peer} (req_id={request_id:?})");
        }
        Err(()) => {
            let decline = S2cMsg::NewGameResponse {
                accepted: Some(false),
                receiver_peer_id: None,
            };
            if swarm
                .behaviour_mut()
                .c2s
                .send_response(channel, decline)
                .is_err()
            {
                log::warn!("Could not send NewGameResponse (decline) to {peer}: channel closed");
            }
        }
    }
}

async fn handle_s2c_event(
    swarm: &mut Swarm<Behaviour>,
    _registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    event: request_response::Event<S2cMsg, C2sMsg>,
) -> Result<(), P2pError> {
    use request_response::{Event, Message};
    match event {
        Event::Message {
            peer: _,
            message:
                Message::Response {
                    request_id,
                    response,
                },
            ..
        } => {
            let Some(pending_game) = pending.remove(&request_id) else {
                log::warn!("Received s2c response for unknown request_id {request_id:?}");
                return Ok(());
            };
            let accepted = match response {
                C2sMsg::NewGameAnswer { accepted } => accepted,
                _ => false,
            };

            let receiver_peer_id = if accepted {
                Some(pending_game.receiver_peer_id.to_string())
            } else {
                None
            };
            let resp = S2cMsg::NewGameResponse { accepted: Some(accepted), receiver_peer_id };
            if swarm
                .behaviour_mut()
                .c2s
                .send_response(pending_game.channel, resp)
                .is_err()
            {
                log::warn!("Could not forward NewGameResponse to original requester");
            }
        }
        Event::OutboundFailure {
            peer,
            request_id,
            error,
            ..
        } => {
            log::warn!("s2c outbound failure to {peer}: {error:?} (id={request_id:?})");
            // Receiver disconnected or timed out — decline the original requester.
            if let Some(pending_game) = pending.remove(&request_id) {
                let decline = S2cMsg::NewGameResponse {
                    accepted: Some(false),
                    receiver_peer_id: None,
                };
                if swarm
                    .behaviour_mut()
                    .c2s
                    .send_response(pending_game.channel, decline)
                    .is_err()
                {
                    log::warn!("Could not send NewGameResponse (outbound failure) to requester");
                }
            }
        }
        Event::InboundFailure {
            peer,
            request_id,
            error,
            ..
        } => {
            log::warn!("s2c inbound failure from {peer}: {error:?} (id={request_id:?})");
        }
        _ => {}
    }
    Ok(())
}

fn validate_jwt(config: &JwtConfig, token: &str) -> Result<Claims, P2pError> {
    let token_data =
        jsonwebtoken::decode::<Claims>(token, &config.jwt_decoding_key, &config.jwt_validation)
            .map_err(|e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => P2pError::TokenExpired,
                _ => P2pError::InvalidToken,
            })?;
    Ok(token_data.claims)
}

// ── Peer registry ────────────────────────────────────────────────────────────

#[derive(Default)]
struct PeerRegistry {
    user_to_peer: DashMap<Uuid, PeerId>,
    peer_to_user: DashMap<PeerId, Uuid>,
}

impl PeerRegistry {
    fn register(&self, user_id: Uuid, peer_id: PeerId) {
        // Remove any stale mapping for this user.
        if let Some((_, old_peer)) = self.user_to_peer.remove(&user_id) {
            self.peer_to_user.remove(&old_peer);
        }
        self.user_to_peer.insert(user_id, peer_id);
        self.peer_to_user.insert(peer_id, user_id);
    }

    fn remove_peer(&self, peer_id: &PeerId) {
        if let Some((_, user_id)) = self.peer_to_user.remove(peer_id) {
            self.user_to_peer.remove(&user_id);
        }
    }

    fn peer_id_for_user(&self, user_id: &Uuid) -> Option<PeerId> {
        self.user_to_peer.get(user_id).map(|r| *r)
    }

    fn user_id_for_peer(&self, peer_id: &PeerId) -> Option<Uuid> {
        self.peer_to_user.get(peer_id).map(|r| *r)
    }
}

// ── Pending new-game requests ────────────────────────────────────────────────

struct PendingNewGame {
    /// Channel to reply to the original NewGame requester (c2s behaviour).
    channel: ResponseChannel<S2cMsg>,
    sender_user_id: Uuid,
    receiver_peer_id: PeerId,
}

// ── Codecs ───────────────────────────────────────────────────────────────────

/// Protocol identifier for client-to-server requests.
#[derive(Debug, Clone)]
struct C2sProtocol;

impl AsRef<str> for C2sProtocol {
    fn as_ref(&self) -> &str {
        "c2s/v1"
    }
}

/// Protocol identifier for server-to-client push requests.
#[derive(Debug, Clone)]
struct S2cProtocol;

impl AsRef<str> for S2cProtocol {
    fn as_ref(&self) -> &str {
        "s2c/v1"
    }
}

/// Codec for client-initiated (c2s) request/response pairs.
#[derive(Debug, Clone, Default)]
struct C2sCodec;

#[async_trait]
impl Codec for C2sCodec {
    type Protocol = C2sProtocol;
    type Request = C2sMsg;
    type Response = S2cMsg;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        C2sMsg::deserialize(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        S2cMsg::deserialize(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&req.serialize()).await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&res.serialize()).await
    }
}

/// Codec for server-initiated (s2c) push request/response pairs.
#[derive(Debug, Clone, Default)]
struct S2cCodec;

#[async_trait]
impl Codec for S2cCodec {
    type Protocol = S2cProtocol;
    type Request = S2cMsg;
    type Response = C2sMsg;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        S2cMsg::deserialize(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        C2sMsg::deserialize(&buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&req.serialize()).await
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        io.write_all(&res.serialize()).await
    }
}

// ── Network behaviour ────────────────────────────────────────────────────────

#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    ping: ping::Behaviour,
    /// Client-initiated requests: client sends Msg, server replies with Msg.
    c2s: request_response::Behaviour<C2sCodec>,
    /// Server-initiated push: server sends NewGameEvent, client replies.
    s2c: request_response::Behaviour<S2cCodec>,
}

impl Behaviour {
    fn create(key: &Keypair) -> Self {
        let mut builder = gossipsub::ConfigBuilder::default();
        builder
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(|message: &gossipsub::Message| {
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            });
        #[cfg(debug_assertions)]
        {
            builder.heartbeat_interval(Duration::from_secs(10));
        }
        let gossipsub_config = builder.build().expect("Could not build gossipsub config");
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )
        .expect("Could not build gossipsub behaviour");

        Behaviour {
            gossipsub,
            ping: ping::Behaviour::new(ping::Config::new()),
            c2s: request_response::Behaviour::new(
                [(C2sProtocol, ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
            s2c: request_response::Behaviour::new(
                [(S2cProtocol, ProtocolSupport::Full)],
                request_response::Config::default(),
            ),
        }
    }
}

// ── Error type ───────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("init-p2p")]
    InitP2p,
    #[error("transport")]
    Transport(#[from] TransportError<io::Error>),
    #[error("db")]
    DbQuery(#[from] sea_orm::DbErr),
    #[error("invalid-token")]
    InvalidToken,
    #[error("token-expired")]
    TokenExpired,
}

impl From<P2pError> for io::Error {
    fn from(value: P2pError) -> Self {
        io::Error::new(io::ErrorKind::Other, value)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
    use uuid::Uuid;

    use crate::api::auth::session::{claims::Claims, config::Config as JwtConfig};

    use super::{validate_jwt, PeerRegistry, P2pError};

    // ── helpers ───────────────────────────────────────────────────────────────

    fn test_jwt_config() -> JwtConfig {
        let secret = b"p2p-test-secret";
        let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        validation.set_audience(&["test"]);
        validation.set_issuer(&["test"]);
        JwtConfig {
            jwt_encoding_key: EncodingKey::from_secret(secret),
            jwt_decoding_key: DecodingKey::from_secret(secret),
            jwt_validation: validation,
            jwt_audience: vec!["test".into()],
            jwt_issuers: vec!["test".into()],
        }
    }

    fn make_token(config: &JwtConfig, user_id: Uuid) -> String {
        Claims::new_24_hours(config, user_id)
            .unwrap()
            .generate_token(config)
            .unwrap()
    }

    fn make_peer_id() -> libp2p::PeerId {
        libp2p::PeerId::from(libp2p::identity::Keypair::generate_ed25519().public())
    }

    // ── PeerRegistry ──────────────────────────────────────────────────────────

    #[test]
    fn register_and_lookup_both_directions() {
        let registry = PeerRegistry::default();
        let user_id = Uuid::new_v4();
        let peer_id = make_peer_id();

        registry.register(user_id, peer_id);

        assert_eq!(registry.peer_id_for_user(&user_id), Some(peer_id));
        assert_eq!(registry.user_id_for_peer(&peer_id), Some(user_id));
    }

    #[test]
    fn register_replaces_stale_mapping_for_same_user() {
        let registry = PeerRegistry::default();
        let user_id = Uuid::new_v4();
        let old_peer = make_peer_id();
        let new_peer = make_peer_id();

        registry.register(user_id, old_peer);
        registry.register(user_id, new_peer);

        // New peer is authoritative.
        assert_eq!(registry.peer_id_for_user(&user_id), Some(new_peer));
        assert_eq!(registry.user_id_for_peer(&new_peer), Some(user_id));
        // Old peer entry must be gone.
        assert_eq!(registry.user_id_for_peer(&old_peer), None);
    }

    #[test]
    fn remove_peer_cleans_both_directions() {
        let registry = PeerRegistry::default();
        let user_id = Uuid::new_v4();
        let peer_id = make_peer_id();

        registry.register(user_id, peer_id);
        registry.remove_peer(&peer_id);

        assert_eq!(registry.peer_id_for_user(&user_id), None);
        assert_eq!(registry.user_id_for_peer(&peer_id), None);
    }

    #[test]
    fn lookup_unknown_returns_none() {
        let registry = PeerRegistry::default();
        let unknown_user = Uuid::new_v4();
        let unknown_peer = make_peer_id();

        assert_eq!(registry.peer_id_for_user(&unknown_user), None);
        assert_eq!(registry.user_id_for_peer(&unknown_peer), None);
    }

    #[test]
    fn remove_peer_on_unknown_is_a_noop() {
        let registry = PeerRegistry::default();
        let unknown_peer = make_peer_id();
        // Should not panic.
        registry.remove_peer(&unknown_peer);
    }

    // ── validate_jwt ──────────────────────────────────────────────────────────

    #[test]
    fn valid_token_returns_correct_claims() {
        let config = test_jwt_config();
        let user_id = Uuid::new_v4();
        let token = make_token(&config, user_id);

        let claims = validate_jwt(&config, &token).expect("should accept valid token");
        assert_eq!(claims.sub, user_id);
    }

    #[test]
    fn tampered_token_is_rejected() {
        let config = test_jwt_config();
        let token = make_token(&config, Uuid::new_v4());
        let tampered = format!("{token}x");

        let err = validate_jwt(&config, &tampered).expect_err("should reject tampered token");
        assert!(matches!(err, P2pError::InvalidToken));
    }

    #[test]
    fn wrong_secret_is_rejected() {
        let config = test_jwt_config();
        let other_config = {
            let secret = b"totally-different-secret";
            let mut v = Validation::new(jsonwebtoken::Algorithm::HS256);
            v.set_audience(&["test"]);
            v.set_issuer(&["test"]);
            JwtConfig {
                jwt_encoding_key: EncodingKey::from_secret(secret),
                jwt_decoding_key: DecodingKey::from_secret(secret),
                jwt_validation: v,
                jwt_audience: vec!["test".into()],
                jwt_issuers: vec!["test".into()],
            }
        };
        // Token signed with `other_config`, verified with `config` — must fail.
        let token = make_token(&other_config, Uuid::new_v4());

        let err = validate_jwt(&config, &token).expect_err("should reject wrong-secret token");
        assert!(matches!(err, P2pError::InvalidToken));
    }

    #[test]
    fn expired_token_returns_token_expired_error() {
        use chrono::{Duration, Utc};
        use jsonwebtoken::Header;

        let config = test_jwt_config();
        let user_id = Uuid::new_v4();
        let now = Utc::now();
        let expired_claims = Claims {
            sub: user_id,
            aud: vec!["test".into()],
            iss: vec!["test".into()],
            iat: now - Duration::days(2),
            exp: now - Duration::days(1),
        };
        let token = jsonwebtoken::encode(
            &Header::new(jsonwebtoken::Algorithm::HS256),
            &expired_claims,
            &config.jwt_encoding_key,
        )
        .unwrap();

        let err = validate_jwt(&config, &token).expect_err("should reject expired token");
        assert!(matches!(err, P2pError::TokenExpired));
    }
}
