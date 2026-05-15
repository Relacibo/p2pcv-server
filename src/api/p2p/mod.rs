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
    Multiaddr, PeerId, Swarm, SwarmBuilder, TransportError,
};
use libp2p_core::transport::Transport;
use libp2p_webrtc::tokio::Certificate;
use p2pcv_protobuf::{client_to_server, server_to_client};
use prost::Message;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    api::auth::session::{claims::Claims, config::Config as JwtConfig},
    db::{db_conn::DbPool, users::User},
};

pub async fn init(pool: DbPool, jwt_config: JwtConfig) -> Result<(), P2pError> {
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
    let private_token =
        env::var("P2P_PRIVATE_KEY_ED25519").expect("P2P_PRIVATE_KEY_ED25519 needed!");
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

    let cert = env::var("P2P_TRANSPORT_CERT_PEM")
        .expect("P2P_TRANSPORT_CERT_PEM needed!")
        .replace("$", "\n");

    let mut swarm = SwarmBuilder::with_existing_identity(keypair)
        .with_tokio()
        .with_other_transport(|id_keys| {
            let transport = libp2p_webrtc::tokio::Transport::new(
                id_keys.clone(),
                Certificate::from_pem(cert.as_str()).expect("pem invalid!"),
            );
            let res = transport.map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)));
            Ok(res)
        })
        .expect("Could not add WebRTC transport")
        .with_behaviour(Behaviour::create)
        .map_err(|_| P2pError::InitP2p)?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(Duration::from_secs(timeout_secs))
        })
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
            handle_swarm_event(&mut swarm, &pool, &jwt_config, &registry, &mut pending, event)
                .await
        {
            log::error!("p2p event error: {err}");
        }
    }

    Ok(())
}

async fn handle_swarm_event(
    swarm: &mut Swarm<Behaviour>,
    pool: &DbPool,
    jwt_config: &JwtConfig,
    registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    event: SwarmEvent<BehaviourEvent>,
) -> Result<(), P2pError> {
    log::debug!("{:?}", event);
    match event {
        SwarmEvent::Behaviour(BehaviourEvent::C2s(ev)) => {
            handle_c2s_event(swarm, pool, jwt_config, registry, pending, ev).await?;
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
    pool: &DbPool,
    jwt_config: &JwtConfig,
    registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    event: request_response::Event<client_to_server::Msg, server_to_client::Msg>,
) -> Result<(), P2pError> {
    use request_response::{Event, Message};
    match event {
        Event::Message {
            peer,
            message: Message::Request { request, channel, .. },
        } => {
            let Some(c2s) = request.c2s else {
                log::warn!("Received empty c2s message from {peer}");
                return Ok(());
            };
            use client_to_server::msg::C2s;
            match c2s {
                C2s::RegisterPeer(reg) => {
                    handle_register_peer(swarm, jwt_config, registry, peer, reg, channel).await;
                }
                C2s::GetFriendPeerId(req) => {
                    handle_get_friend_peer_id(swarm, pool, registry, peer, req, channel).await;
                }
                C2s::NewGame(req) => {
                    handle_new_game(swarm, pool, registry, pending, peer, req, channel).await;
                }
                C2s::NewGameAnswer(_) => {
                    log::warn!("Unexpected NewGameAnswer via c2s from {peer}; ignoring");
                }
            }
        }
        Event::OutboundFailure { peer, request_id, error } => {
            log::warn!("c2s outbound failure to {peer}: {error:?} (id={request_id:?})");
        }
        Event::InboundFailure { peer, request_id, error } => {
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
    reg: client_to_server::RegisterPeer,
    channel: ResponseChannel<server_to_client::Msg>,
) {
    let resp = match validate_jwt(jwt_config, &reg.auth_token) {
        Ok(claims) => {
            let user_id = claims.sub;
            registry.register(user_id, peer);
            log::info!("Registered peer {peer} for user {user_id}");
            server_to_client::Msg {
                s2c: Some(server_to_client::msg::S2c::RegisterPeerResponse(
                    server_to_client::RegisterPeerResponse { success: true },
                )),
            }
        }
        Err(err) => {
            log::warn!("RegisterPeer auth failed for {peer}: {err}");
            server_to_client::Msg {
                s2c: Some(server_to_client::msg::S2c::RegisterPeerResponse(
                    server_to_client::RegisterPeerResponse { success: false },
                )),
            }
        }
    };
    if swarm.behaviour_mut().c2s.send_response(channel, resp).is_err() {
        log::warn!("Could not send RegisterPeerResponse to {peer}: channel closed");
    }
}

async fn handle_get_friend_peer_id(
    swarm: &mut Swarm<Behaviour>,
    pool: &DbPool,
    registry: &PeerRegistry,
    peer: PeerId,
    req: client_to_server::GetFriendPeerId,
    channel: ResponseChannel<server_to_client::Msg>,
) {
    let peer_id_str = async {
        let requester_user_id = registry.user_id_for_peer(&peer)?;
        let friend_user_id =
            Uuid::from_slice(&req.friend_user_id).ok().filter(|id| !id.is_nil())?;

        let mut db = pool.get().await.ok()?;
        let is_friends = User::is_friends_with(&mut *db, requester_user_id, friend_user_id)
            .await
            .ok()?;
        if !is_friends {
            return None;
        }
        let friend_peer = registry.peer_id_for_user(&friend_user_id)?;
        Some(friend_peer.to_string())
    }
    .await;

    let resp = server_to_client::Msg {
        s2c: Some(server_to_client::msg::S2c::GetFriendPeerIdResponse(
            server_to_client::GetFriendPeerIdResponse {
                peer_id: peer_id_str,
            },
        )),
    };
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
    pool: &DbPool,
    registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    peer: PeerId,
    req: client_to_server::NewGame,
    channel: ResponseChannel<server_to_client::Msg>,
) {
    let result: Result<(PeerId, Uuid, server_to_client::NewGameEvent), ()> = async {
        let sender_user_id = registry.user_id_for_peer(&peer).ok_or(())?;

        let receiver_user_id =
            Uuid::from_slice(&req.receiver_user_id).ok().filter(|id| !id.is_nil()).ok_or(())?;

        let mut db = pool.get().await.map_err(|_| ())?;
        let is_friends = User::is_friends_with(&mut *db, sender_user_id, receiver_user_id)
            .await
            .map_err(|_| ())?;
        if !is_friends {
            return Err(());
        }

        let receiver_peer = registry.peer_id_for_user(&receiver_user_id).ok_or(())?;

        let sender_user = User::get(&mut *db, sender_user_id).await.map_err(|_| ())?;

        let event = server_to_client::NewGameEvent {
            sender_user_id: sender_user_id.as_bytes().to_vec(),
            sender_user_name: sender_user.user_name,
            variant_id: req.variant_id,
            variant_version: req.variant_version,
            timeout_secs: 60,
        };
        Ok((receiver_peer, sender_user_id, event))
    }
    .await;

    match result {
        Ok((receiver_peer, sender_user_id, event)) => {
            let msg = server_to_client::Msg {
                s2c: Some(server_to_client::msg::S2c::NewGameEvent(event)),
            };
            let request_id =
                swarm.behaviour_mut().s2c.send_request(&receiver_peer, msg);
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
            let decline = server_to_client::Msg {
                s2c: Some(server_to_client::msg::S2c::NewGameResponse(
                    server_to_client::NewGameResponse {
                        accepted: false,
                        receiver_peer_id: None,
                    },
                )),
            };
            if swarm.behaviour_mut().c2s.send_response(channel, decline).is_err() {
                log::warn!("Could not send NewGameResponse (decline) to {peer}: channel closed");
            }
        }
    }
}

async fn handle_s2c_event(
    swarm: &mut Swarm<Behaviour>,
    registry: &PeerRegistry,
    pending: &mut HashMap<OutboundRequestId, PendingNewGame>,
    event: request_response::Event<server_to_client::Msg, client_to_server::Msg>,
) -> Result<(), P2pError> {
    use request_response::{Event, Message};
    match event {
        Event::Message {
            peer,
            message: Message::Response { request_id, response },
        } => {
            let Some(pending_game) = pending.remove(&request_id) else {
                log::warn!("Received s2c response for unknown request_id {request_id:?}");
                return Ok(());
            };
            let accepted = response
                .c2s
                .and_then(|c2s| {
                    if let client_to_server::msg::C2s::NewGameAnswer(a) = c2s {
                        Some(a.accepted)
                    } else {
                        None
                    }
                })
                .unwrap_or(false);

            let receiver_peer_id = if accepted {
                Some(pending_game.receiver_peer_id.to_string())
            } else {
                None
            };
            let resp = server_to_client::Msg {
                s2c: Some(server_to_client::msg::S2c::NewGameResponse(
                    server_to_client::NewGameResponse {
                        accepted,
                        receiver_peer_id,
                    },
                )),
            };
            if swarm
                .behaviour_mut()
                .c2s
                .send_response(pending_game.channel, resp)
                .is_err()
            {
                log::warn!("Could not forward NewGameResponse to original requester");
            }
        }
        Event::OutboundFailure { peer, request_id, error } => {
            log::warn!("s2c outbound failure to {peer}: {error:?} (id={request_id:?})");
            // Receiver disconnected or timed out — decline the original requester.
            if let Some(pending_game) = pending.remove(&request_id) {
                let decline = server_to_client::Msg {
                    s2c: Some(server_to_client::msg::S2c::NewGameResponse(
                        server_to_client::NewGameResponse {
                            accepted: false,
                            receiver_peer_id: None,
                        },
                    )),
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
        Event::InboundFailure { peer, request_id, error } => {
            log::warn!("s2c inbound failure from {peer}: {error:?} (id={request_id:?})");
        }
        _ => {}
    }
    Ok(())
}

fn validate_jwt(config: &JwtConfig, token_bytes: &[u8]) -> Result<Claims, P2pError> {
    let token_str = std::str::from_utf8(token_bytes).map_err(|_| P2pError::InvalidToken)?;
    let token_data =
        jsonwebtoken::decode::<Claims>(token_str, &config.jwt_decoding_key, &config.jwt_validation)
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
    channel: ResponseChannel<server_to_client::Msg>,
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
    type Request = client_to_server::Msg;
    type Response = server_to_client::Msg;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        client_to_server::Msg::decode(&*buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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
        server_to_client::Msg::decode(&*buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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
        let mut buf = Vec::new();
        req.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await
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
        let mut buf = Vec::new();
        res.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await
    }
}

/// Codec for server-initiated (s2c) push request/response pairs.
#[derive(Debug, Clone, Default)]
struct S2cCodec;

#[async_trait]
impl Codec for S2cCodec {
    type Protocol = S2cProtocol;
    type Request = server_to_client::Msg;
    type Response = client_to_server::Msg;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        server_to_client::Msg::decode(&*buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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
        client_to_server::Msg::decode(&*buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
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
        let mut buf = Vec::new();
        req.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await
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
        let mut buf = Vec::new();
        res.encode(&mut buf)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        io.write_all(&buf).await
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
    #[error("db-pool")]
    DbPool,
    #[error("db")]
    DbQuery(#[from] diesel::result::Error),
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
