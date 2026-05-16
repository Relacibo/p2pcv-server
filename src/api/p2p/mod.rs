use std::{
    collections::HashMap,
    env,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    net::Ipv4Addr,
    str::FromStr,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use dashmap::DashMap;
use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, StreamExt};
use libp2p::{
    core::muxing::StreamMuxerBox,
    gossipsub, identify,
    identity::Keypair,
    multiaddr::Protocol,
    ping, relay,
    request_response::{self, Codec, ProtocolSupport, ResponseChannel},
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

pub struct P2pInfo {
    pub peer_id: PeerId,
    pub multiaddr: Multiaddr,
}

fn read_keypair() -> Keypair {
    let private_token = crate::secret::read_secret("P2P_PRIVATE_KEY_ED25519");
    let mut private_key = BASE64_STANDARD
        .decode(private_token)
        .expect("P2P_PRIVATE_KEY_ED25519 is invalid base64");
    private_key = private_key.into_iter().rev().take(32).rev().collect::<Vec<_>>();
    Keypair::ed25519_from_bytes(private_key).expect("P2P_PRIVATE_KEY_ED25519 is not a private key")
}

/// Returns the P2P node info (multiaddr with certhash) and the WebRTC certificate.
/// The certificate must be passed to `init()` so both share the same cert/certhash.
pub fn p2p_info() -> (P2pInfo, Certificate) {
    let keypair = read_keypair();
    let peer_id = keypair.public().to_peer_id();
    // P2P_EXTERNAL_IP is the public IP advertised to clients.
    // Falls back to P2P_HOST if not set (useful for local dev).
    let external_host = env::var("P2P_EXTERNAL_IP")
        .or_else(|_| env::var("P2P_HOST"))
        .expect("P2P_HOST needed!");
    let external_address = Ipv4Addr::from_str(&external_host)
        .expect("Invalid P2P_EXTERNAL_IP / P2P_HOST address");
    let port: u16 = env::var("P2P_PORT")
        .expect("P2P_PORT needed!")
        .parse()
        .expect("P2P_PORT not a number");
    let cert = Certificate::generate(&mut rand::thread_rng())
        .expect("Failed to generate WebRTC certificate");
    let certhash = cert.fingerprint().to_multihash();
    let multiaddr = Multiaddr::from(external_address)
        .with(Protocol::Udp(port))
        .with(Protocol::WebRTCDirect)
        .with(Protocol::Certhash(certhash))
        .with(Protocol::P2p(peer_id));
    (P2pInfo { peer_id, multiaddr }, cert)
}

pub async fn init(db: DbPool, jwt_config: JwtConfig, cert: Certificate) -> Result<(), P2pError> {
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

    let keypair = read_keypair();

    let local_peer_id = keypair.public().to_peer_id();
    let mut swarm = SwarmBuilder::with_existing_identity(keypair.clone())
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
        .with_behaviour(|key| Behaviour::create(key, local_peer_id))
        .map_err(|_| P2pError::InitP2p)?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(timeout_secs)))
        .build();

    let address_webrtc = Multiaddr::from(address)
        .with(Protocol::Udp(port))
        .with(Protocol::WebRTCDirect);

    swarm.listen_on(address_webrtc)?;

    let registry = PeerRegistry::default();
    let mut lobbies: LobbyRegistry = HashMap::new();
    let mut heartbeat_check = tokio::time::interval(Duration::from_secs(10));
    // Skip the first immediate tick.
    heartbeat_check.tick().await;

    loop {
        tokio::select! {
            event = swarm.select_next_some() => {
                if let Err(err) =
                    handle_swarm_event(&mut swarm, &db, &jwt_config, &registry, &mut lobbies, event).await
                {
                    log::error!("p2p event error: {err}");
                }
            }
            _ = heartbeat_check.tick() => {
                check_lobby_heartbeats(&mut swarm, &registry, &mut lobbies);
            }
            _ = tokio::signal::ctrl_c() => break,
        }
    }

    Ok(())
}

async fn handle_swarm_event(
    swarm: &mut Swarm<Behaviour>,
    db: &DbPool,
    jwt_config: &JwtConfig,
    registry: &PeerRegistry,
    lobbies: &mut LobbyRegistry,
    event: SwarmEvent<BehaviourEvent>,
) -> Result<(), P2pError> {
    log::debug!("{:?}", event);
    match event {
        SwarmEvent::Behaviour(BehaviourEvent::C2s(ev)) => {
            handle_c2s_event(swarm, db, jwt_config, registry, lobbies, ev).await?;
        }
        SwarmEvent::Behaviour(BehaviourEvent::S2c(ev)) => {
            handle_s2c_event(ev);
        }
        SwarmEvent::Behaviour(BehaviourEvent::Relay(ev)) => {
            log::debug!("Relay event: {:?}", ev);
        }
        SwarmEvent::Behaviour(BehaviourEvent::Identify(ev)) => {
            log::debug!("Identify event: {:?}", ev);
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
            peer_left_all_lobbies(swarm, lobbies, &peer_id);
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
    lobbies: &mut LobbyRegistry,
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
            match request {
                C2sMsg::RegisterPeer { auth_token } => {
                    handle_register_peer(swarm, db, jwt_config, registry, peer, auth_token, channel)
                        .await;
                }
                C2sMsg::GetFriendPeerId { friend_user_id } => {
                    handle_get_friend_peer_id(swarm, db, registry, peer, friend_user_id, channel)
                        .await;
                }
                C2sMsg::CreateLobby { variant_id, variant_version, script_url } => {
                    handle_create_lobby(swarm, registry, lobbies, peer, variant_id, variant_version, script_url, channel).await;
                }
                C2sMsg::InviteToLobby { lobby_id, friend_user_ids } => {
                    handle_invite_to_lobby(swarm, db, registry, lobbies, peer, lobby_id, friend_user_ids, channel).await;
                }
                C2sMsg::JoinLobby { lobby_id } => {
                    handle_join_lobby(swarm, registry, lobbies, peer, lobby_id, channel).await;
                }
                C2sMsg::LeaveLobby { lobby_id } => {
                    handle_leave_lobby(swarm, lobbies, peer, lobby_id, channel).await;
                }
                C2sMsg::DeleteLobby { lobby_id } => {
                    handle_delete_lobby(swarm, lobbies, peer, lobby_id, channel).await;
                }
                C2sMsg::StartGame { lobby_id } => {
                    handle_start_game(swarm, lobbies, peer, lobby_id, channel).await;
                }
                C2sMsg::GameEnded { lobby_id } => {
                    handle_game_ended(swarm, lobbies, peer, lobby_id, channel).await;
                }
                C2sMsg::LobbyHeartbeat { lobby_id } => {
                    handle_lobby_heartbeat(swarm, lobbies, peer, lobby_id, channel).await;
                }
                C2sMsg::LobbyEventAck {} => {
                    // Ack for a server-pushed lobby event; nothing to do.
                }
                C2sMsg::Unknown => {
                    log::warn!("Received unknown C2S message variant from {peer}; ignoring");
                }
            }
        }
        Event::OutboundFailure { peer, request_id, error, .. } => {
            log::warn!("c2s outbound failure to {peer}: {error:?} (id={request_id:?})");
        }
        Event::InboundFailure { peer, request_id, error, .. } => {
            log::warn!("c2s inbound failure from {peer}: {error:?} (id={request_id:?})");
        }
        _ => {}
    }
    Ok(())
}

async fn handle_register_peer(
    swarm: &mut Swarm<Behaviour>,
    db: &DbPool,
    jwt_config: &JwtConfig,
    registry: &PeerRegistry,
    peer: PeerId,
    auth_token: String,
    channel: ResponseChannel<S2cMsg>,
) {
    let resp = match validate_jwt(jwt_config, &auth_token) {
        Ok(claims) => {
            let user_id = claims.sub;
            let display_name = User::get(db, user_id)
                .await
                .map(|u| u.user_name)
                .unwrap_or_default();
            registry.register(user_id, peer, display_name);
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

// ── Lobby handlers ───────────────────────────────────────────────────────────

async fn handle_create_lobby(
    swarm: &mut Swarm<Behaviour>,
    registry: &PeerRegistry,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    variant_id: Guid,
    variant_version: String,
    script_url: String,
    channel: ResponseChannel<S2cMsg>,
) {
    let Some(host_user_id) = registry.user_id_for_peer(&peer) else {
        log::warn!("CreateLobby from unregistered peer {peer}; ignoring");
        send_response(swarm, channel, S2cMsg::CreateLobbyResponse { success: Some(false), lobby_id: None }, &peer);
        return;
    };
    let display_name = registry.display_name_for_peer(&peer).unwrap_or_default();
    let lobby_id = Uuid::new_v4();
    let member = LobbyMember {
        user_id: host_user_id,
        display_name,
        last_heartbeat: Instant::now(),
    };
    let mut members = HashMap::new();
    members.insert(peer, member);
    lobbies.insert(
        lobby_id,
        Lobby {
            host_peer_id: peer,
            host_user_id,
            status: LobbyStatus::Waiting,
            variant_id,
            variant_version,
            script_url,
            members,
        },
    );
    log::info!("Lobby {lobby_id} created by {peer}");
    send_response(swarm, channel, S2cMsg::CreateLobbyResponse { success: Some(true), lobby_id: Some(lobby_id.to_guid()) }, &peer);
}

async fn handle_invite_to_lobby(
    swarm: &mut Swarm<Behaviour>,
    db: &DbPool,
    registry: &PeerRegistry,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    lobby_id: Guid,
    friend_user_ids: Vec<Guid>,
    channel: ResponseChannel<S2cMsg>,
) {
    let result: Result<(), ()> = async {
        let lobby_id = lobby_id.to_uuid();
        let lobby = lobbies.get(&lobby_id).ok_or(())?;
        if lobby.host_peer_id != peer {
            return Err(());
        }
        let requester_user_id = registry.user_id_for_peer(&peer).ok_or(())?;
        let host_name = registry.display_name_for_peer(&peer).unwrap_or_default();

        for friend_user_id_guid in &friend_user_ids {
            let friend_user_id = friend_user_id_guid.to_uuid();
            if friend_user_id.is_nil() {
                continue;
            }
            let is_friends = User::is_friends_with(db, requester_user_id, friend_user_id)
                .await
                .unwrap_or(false);
            if !is_friends {
                continue;
            }
            let Some(friend_peer) = registry.peer_id_for_user(&friend_user_id) else {
                continue;
            };
            let invite = S2cMsg::LobbyInvite {
                lobby_id: Some(lobby_id.to_guid()),
                host_user_id: Some(requester_user_id.to_guid()),
                host_name: Some(host_name.clone()),
                variant_id: Some(lobby.variant_id),
                variant_version: Some(lobby.variant_version.clone()),
                script_url: Some(lobby.script_url.clone()),
            };
            swarm.behaviour_mut().s2c.send_request(&friend_peer, invite);
        }
        Ok(())
    }.await;

    let success = result.is_ok();
    send_response(swarm, channel, S2cMsg::InviteToLobbyResponse { success: Some(success) }, &peer);
}

async fn handle_join_lobby(
    swarm: &mut Swarm<Behaviour>,
    registry: &PeerRegistry,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    lobby_id: Guid,
    channel: ResponseChannel<S2cMsg>,
) {
    let lobby_id = lobby_id.to_uuid();
    let result: Result<S2cMsg, ()> = async {
        let Some(user_id) = registry.user_id_for_peer(&peer) else {
            return Err(());
        };
        let display_name = registry.display_name_for_peer(&peer).unwrap_or_default();
        let lobby = lobbies.get_mut(&lobby_id).ok_or(())?;

        let member = LobbyMember {
            user_id,
            display_name: display_name.clone(),
            last_heartbeat: Instant::now(),
        };
        lobby.members.insert(peer, member);

        let member_peer_ids: Vec<String> = lobby.members.keys().map(|p| p.to_string()).collect();
        let member_names: Vec<String> = lobby.members.values().map(|m| m.display_name.clone()).collect();
        let member_user_ids: Vec<Guid> = lobby.members.values().map(|m| m.user_id.to_guid()).collect();
        let host_peer_id = lobby.host_peer_id.to_string();
        let in_game = lobby.status == LobbyStatus::InGame;

        Ok(S2cMsg::JoinLobbyResponse {
            success: Some(true),
            member_peer_ids: Some(member_peer_ids),
            member_names: Some(member_names),
            member_user_ids: Some(member_user_ids),
            host_peer_id: Some(host_peer_id),
            in_game: Some(in_game),
        })
    }.await;

    match result {
        Ok(resp) => {
            // Notify existing members about the new joiner.
            let join_event = S2cMsg::LobbyMemberJoined {
                lobby_id: Some(lobby_id.to_guid()),
                peer_id: Some(peer.to_string()),
                user_id: Some(registry.user_id_for_peer(&peer).unwrap_or_default().to_guid()),
                display_name: Some(registry.display_name_for_peer(&peer).unwrap_or_default()),
            };
            push_to_lobby_members(swarm, lobbies, lobby_id, join_event, Some(&peer));
            send_response(swarm, channel, resp, &peer);
        }
        Err(()) => {
            send_response(swarm, channel, S2cMsg::JoinLobbyResponse {
                success: Some(false),
                member_peer_ids: None,
                member_names: None,
                member_user_ids: None,
                host_peer_id: None,
                in_game: None,
            }, &peer);
        }
    }
}

async fn handle_leave_lobby(
    swarm: &mut Swarm<Behaviour>,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    lobby_id: Guid,
    channel: ResponseChannel<S2cMsg>,
) {
    let lobby_id = lobby_id.to_uuid();
    remove_peer_from_lobby(swarm, lobbies, lobby_id, &peer);
    send_response(swarm, channel, S2cMsg::LeaveLobbyResponse {}, &peer);
}

async fn handle_delete_lobby(
    swarm: &mut Swarm<Behaviour>,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    lobby_id: Guid,
    channel: ResponseChannel<S2cMsg>,
) {
    let lobby_id = lobby_id.to_uuid();
    let success = lobbies
        .get(&lobby_id)
        .map(|l| l.host_peer_id == peer)
        .unwrap_or(false);

    if success {
        let deleted = S2cMsg::LobbyDeleted { lobby_id: Some(lobby_id.to_guid()) };
        push_to_lobby_members(swarm, lobbies, lobby_id, deleted, None);
        lobbies.remove(&lobby_id);
        log::info!("Lobby {lobby_id} deleted by host {peer}");
    } else {
        log::warn!("DeleteLobby for {lobby_id} denied or not found (peer={peer})");
    }
    send_response(swarm, channel, S2cMsg::DeleteLobbyResponse { success: Some(success) }, &peer);
}

async fn handle_start_game(
    swarm: &mut Swarm<Behaviour>,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    lobby_id: Guid,
    channel: ResponseChannel<S2cMsg>,
) {
    let lobby_id = lobby_id.to_uuid();
    let success = lobbies
        .get(&lobby_id)
        .map(|l| l.host_peer_id == peer)
        .unwrap_or(false);

    if success {
        let peer_ids: Vec<String> = lobbies[&lobby_id].members.keys().map(|p| p.to_string()).collect();
        if let Some(lobby) = lobbies.get_mut(&lobby_id) {
            lobby.status = LobbyStatus::InGame;
        }
        let started = S2cMsg::LobbyGameStarted {
            lobby_id: Some(lobby_id.to_guid()),
            peer_ids: Some(peer_ids),
        };
        push_to_lobby_members(swarm, lobbies, lobby_id, started, None);
        log::info!("Lobby {lobby_id} game started by {peer}");
    } else {
        log::warn!("StartGame for {lobby_id} denied or not found (peer={peer})");
    }
    send_response(swarm, channel, S2cMsg::StartGameResponse { success: Some(success) }, &peer);
}

async fn handle_game_ended(
    swarm: &mut Swarm<Behaviour>,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    lobby_id: Guid,
    channel: ResponseChannel<S2cMsg>,
) {
    let lobby_id = lobby_id.to_uuid();
    if let Some(lobby) = lobbies.get_mut(&lobby_id) {
        if lobby.members.contains_key(&peer) {
            lobby.status = LobbyStatus::Waiting;
            let ended = S2cMsg::LobbyGameEnded { lobby_id: Some(lobby_id.to_guid()) };
            push_to_lobby_members(swarm, lobbies, lobby_id, ended, None);
            log::info!("Lobby {lobby_id} game ended (reported by {peer})");
        }
    }
    send_response(swarm, channel, S2cMsg::GameEndedResponse {}, &peer);
}

async fn handle_lobby_heartbeat(
    swarm: &mut Swarm<Behaviour>,
    lobbies: &mut LobbyRegistry,
    peer: PeerId,
    lobby_id: Guid,
    channel: ResponseChannel<S2cMsg>,
) {
    let lobby_id = lobby_id.to_uuid();
    if let Some(lobby) = lobbies.get_mut(&lobby_id) {
        if let Some(member) = lobby.members.get_mut(&peer) {
            member.last_heartbeat = Instant::now();
        }
    }
    send_response(swarm, channel, S2cMsg::LobbyHeartbeatResponse {}, &peer);
}

fn handle_s2c_event(event: request_response::Event<S2cMsg, C2sMsg>) {
    use request_response::Event;
    match event {
        Event::OutboundFailure { peer, request_id, error, .. } => {
            log::warn!("s2c outbound failure to {peer}: {error:?} (id={request_id:?})");
        }
        Event::InboundFailure { peer, request_id, error, .. } => {
            log::warn!("s2c inbound failure from {peer}: {error:?} (id={request_id:?})");
        }
        _ => {}
    }
}

// ── Lobby helpers ─────────────────────────────────────────────────────────────

fn push_to_lobby_members(
    swarm: &mut Swarm<Behaviour>,
    lobbies: &LobbyRegistry,
    lobby_id: Uuid,
    msg: S2cMsg,
    exclude: Option<&PeerId>,
) {
    let Some(lobby) = lobbies.get(&lobby_id) else { return };
    let peers: Vec<PeerId> = lobby
        .members
        .keys()
        .filter(|p| exclude.map_or(true, |ex| *p != ex))
        .copied()
        .collect();
    for peer_id in peers {
        swarm.behaviour_mut().s2c.send_request(&peer_id, msg.clone());
    }
}

fn send_response(swarm: &mut Swarm<Behaviour>, channel: ResponseChannel<S2cMsg>, msg: S2cMsg, peer: &PeerId) {
    if swarm.behaviour_mut().c2s.send_response(channel, msg).is_err() {
        log::warn!("Could not send response to {peer}: channel closed");
    }
}

/// Called when a peer disconnects: removes them from every lobby they were in.
fn peer_left_all_lobbies(swarm: &mut Swarm<Behaviour>, lobbies: &mut LobbyRegistry, peer: &PeerId) {
    let lobby_ids: Vec<Uuid> = lobbies
        .iter()
        .filter(|(_, l)| l.members.contains_key(peer))
        .map(|(id, _)| *id)
        .collect();
    for lobby_id in lobby_ids {
        remove_peer_from_lobby(swarm, lobbies, lobby_id, peer);
    }
}

/// Removes a peer from a lobby, notifies remaining members, and deletes the lobby if empty.
fn remove_peer_from_lobby(
    swarm: &mut Swarm<Behaviour>,
    lobbies: &mut LobbyRegistry,
    lobby_id: Uuid,
    peer: &PeerId,
) {
    let Some(lobby) = lobbies.get_mut(&lobby_id) else { return };
    if lobby.members.remove(peer).is_none() {
        return;
    }
    log::debug!("Peer {peer} left lobby {lobby_id}");

    if lobby.members.is_empty() {
        lobbies.remove(&lobby_id);
        log::info!("Lobby {lobby_id} auto-deleted (empty)");
        return;
    }

    // Notify remaining members.
    let left = S2cMsg::LobbyMemberLeft {
        lobby_id: Some(lobby_id.to_guid()),
        peer_id: Some(peer.to_string()),
    };
    push_to_lobby_members(swarm, lobbies, lobby_id, left, None);

    // If the host left, reassign to first remaining member.
    let lobby = lobbies.get_mut(&lobby_id).unwrap();
    if lobby.host_peer_id == *peer {
        if let Some((&new_host, _)) = lobby.members.iter().next() {
            lobby.host_peer_id = new_host;
            log::info!("Lobby {lobby_id} host reassigned to {new_host}");
        }
    }
}

/// Periodic heartbeat cleanup: remove members who haven't sent a heartbeat recently.
fn check_lobby_heartbeats(
    swarm: &mut Swarm<Behaviour>,
    _registry: &PeerRegistry,
    lobbies: &mut LobbyRegistry,
) {
    let timeout = Duration::from_secs(30);
    let now = Instant::now();
    let lobby_ids: Vec<Uuid> = lobbies.keys().copied().collect();
    for lobby_id in lobby_ids {
        let stale: Vec<PeerId> = lobbies
            .get(&lobby_id)
            .map(|l| {
                l.members
                    .iter()
                    .filter(|(_, m)| now.duration_since(m.last_heartbeat) > timeout)
                    .map(|(p, _)| *p)
                    .collect()
            })
            .unwrap_or_default();
        for peer in stale {
            log::info!("Lobby {lobby_id}: removing {peer} due to heartbeat timeout");
            remove_peer_from_lobby(swarm, lobbies, lobby_id, &peer);
        }
    }
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
    peer_to_name: DashMap<PeerId, String>,
}

impl PeerRegistry {
    fn register(&self, user_id: Uuid, peer_id: PeerId, display_name: String) {
        // Remove any stale mapping for this user.
        if let Some((_, old_peer)) = self.user_to_peer.remove(&user_id) {
            self.peer_to_user.remove(&old_peer);
            self.peer_to_name.remove(&old_peer);
        }
        self.user_to_peer.insert(user_id, peer_id);
        self.peer_to_user.insert(peer_id, user_id);
        self.peer_to_name.insert(peer_id, display_name);
    }

    fn remove_peer(&self, peer_id: &PeerId) {
        if let Some((_, user_id)) = self.peer_to_user.remove(peer_id) {
            self.user_to_peer.remove(&user_id);
        }
        self.peer_to_name.remove(peer_id);
    }

    fn peer_id_for_user(&self, user_id: &Uuid) -> Option<PeerId> {
        self.user_to_peer.get(user_id).map(|r| *r)
    }

    fn user_id_for_peer(&self, peer_id: &PeerId) -> Option<Uuid> {
        self.peer_to_user.get(peer_id).map(|r| *r)
    }

    fn display_name_for_peer(&self, peer_id: &PeerId) -> Option<String> {
        self.peer_to_name.get(peer_id).map(|r| r.clone())
    }
}

// ── Lobby data structures ────────────────────────────────────────────────────

#[derive(Debug, PartialEq)]
enum LobbyStatus {
    Waiting,
    InGame,
}

struct LobbyMember {
    user_id: Uuid,
    display_name: String,
    last_heartbeat: Instant,
}

struct Lobby {
    host_peer_id: PeerId,
    host_user_id: Uuid,
    status: LobbyStatus,
    variant_id: Guid,
    variant_version: String,
    script_url: String,
    members: HashMap<PeerId, LobbyMember>,
}

type LobbyRegistry = HashMap<Uuid, Lobby>;

// ── Codecs ───────────────────────────────────────────────────────────────────

/// Protocol identifier for client-to-server requests.
#[derive(Debug, Clone)]
struct C2sProtocol;

impl AsRef<str> for C2sProtocol {
    fn as_ref(&self) -> &str {
        "/c2s/v1"
    }
}

/// Protocol identifier for server-to-client push requests.
#[derive(Debug, Clone)]
struct S2cProtocol;

impl AsRef<str> for S2cProtocol {
    fn as_ref(&self) -> &str {
        "/s2c/v1"
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
    /// Server-initiated push: server sends lobby events, client replies.
    s2c: request_response::Behaviour<S2cCodec>,
    /// Circuit relay server: allows clients to use the server as a relay for NAT traversal.
    relay: relay::Behaviour,
    /// Identify protocol: needed for circuit relay to work correctly.
    identify: identify::Behaviour,
}

impl Behaviour {
    fn create(key: &Keypair, local_peer_id: PeerId) -> Self {
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
            relay: relay::Behaviour::new(local_peer_id, relay::Config::default()),
            identify: identify::Behaviour::new(
                identify::Config::new("/p2pcv/1.0.0".into(), key.public()),
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

// ── REST API ──────────────────────────────────────────────────────────────────

use axum::{extract::State, routing::get, Json, Router};
use std::sync::Arc;

#[derive(serde::Serialize)]
pub struct P2pInfoResponse {
    pub peer_id: String,
    pub multiaddr: String,
}

pub fn router() -> Router<Arc<crate::AppState>> {
    Router::new().route("/info", get(get_p2p_info))
}

async fn get_p2p_info(State(state): State<Arc<crate::AppState>>) -> Json<P2pInfoResponse> {
    Json(P2pInfoResponse {
        peer_id: state.p2p_info.peer_id.to_string(),
        multiaddr: state.p2p_info.multiaddr.to_string(),
    })
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

        registry.register(user_id, peer_id, String::new());

        assert_eq!(registry.peer_id_for_user(&user_id), Some(peer_id));
        assert_eq!(registry.user_id_for_peer(&peer_id), Some(user_id));
    }

    #[test]
    fn register_replaces_stale_mapping_for_same_user() {
        let registry = PeerRegistry::default();
        let user_id = Uuid::new_v4();
        let old_peer = make_peer_id();
        let new_peer = make_peer_id();

        registry.register(user_id, old_peer, String::new());
        registry.register(user_id, new_peer, String::new());

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

        registry.register(user_id, peer_id, String::new());
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
