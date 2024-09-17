use std::{
    env,
    hash::{DefaultHasher, Hash, Hasher},
    io::Read,
    net::Ipv4Addr,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use base64::Engine;
use dotenvy::dotenv;
use env_logger::Env;
use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use libp2p::{
    gossipsub,
    identity::Keypair,
    multiaddr::Protocol,
    multihash::Multihash,
    ping,
    swarm::{NetworkBehaviour, SwarmEvent},
};
use libp2p::{Multiaddr, Transport};
use libp2p_core::muxing::StreamMuxerBox;
use p2pcv_protobuf::{
    client_to_server::{self, msg::C2s, NewGame},
    server_to_client::{self, msg::S2c, NewGameEvent},
};
use prost::Message;
use rand::thread_rng;
use tokio::{net::TcpStream, sync::Mutex};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(debug_assertions)]
    dotenvy::from_filename("../../.env").ok();
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let timeout_secs = env::var("P2P_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);

    let server_listen_address =
        env::var("P2P_SERVER_LISTEN_ADDRESS").expect("P2P_SERVER_LISTEN_ADDRESS missing!");

    let receiver_user_id = uuid::uuid!("12345678-dead-beef-b116-b0000000b135");
    let variant_id = uuid::uuid!("fa21a473-dead-beef-b116-b0000000b135");

    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_other_transport(|id_keys| {
            let transport = libp2p_webrtc::tokio::Transport::new(
                id_keys.clone(),
                libp2p_webrtc::tokio::Certificate::generate(&mut thread_rng())
                    .expect("Could not generate certificate"),
            );

            let res = transport.map(|(peer_id, conn), _| (peer_id, StreamMuxerBox::new(conn)));
            Ok(res)
        })?
        .with_behaviour(Behaviour::create)?
        .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(timeout_secs)))
        .build();

    let address_webrtc = Multiaddr::from(Ipv4Addr::UNSPECIFIED)
        .with(Protocol::Udp(0))
        .with(Protocol::WebRTCDirect);

    swarm.listen_on(address_webrtc)?;

    let server_address = server_listen_address
        .parse::<Multiaddr>()
        .expect("Could not parse server multiaddress!");
    swarm.dial(server_address)?;

    loop {
        tokio::select! {
            swarm_event = swarm.next() => {
                if let Some(swarm_event) = swarm_event {
                    match swarm_event {
                        SwarmEvent::ConnectionEstablished { peer_id, connection_id, endpoint, num_established, concurrent_dial_errors, established_in } => {
                            log::info!("Connected to {peer_id}!")
                        },
                        SwarmEvent::ConnectionClosed { peer_id, connection_id, endpoint, num_established, cause } => {
                            log::info!("Connection to {peer_id} closed!")
                        },
                        _ => ()
                    }
                }
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    let new_game_request = NewGame {
        receiver_user_id: receiver_user_id.as_bytes().to_vec(),
        variant_id: variant_id.as_bytes().to_vec(),
        variant_version: "1.0.0".to_owned(),
    };
    let request = C2s::NewGame(new_game_request);

    // pin_mut!(stdin_to_ws, handle);
    // future::select(future1, handle).await;
    Ok(())
}

// We create a custom network behaviour that combines Gossipsub.
#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    ping: ping::Behaviour,
}

impl Behaviour {
    fn create(key: &Keypair) -> Self {
        // Set a custom gossipsub configuration
        let mut builder = gossipsub::ConfigBuilder::default();
        builder
            // This sets the kind of message validation. The default is Strict (enforce message signing)
            .validation_mode(gossipsub::ValidationMode::Strict)
            // content-address messages. No two messages of the same content will be propagated.
            .message_id_fn(|message: &gossipsub::Message| {
                // To content-address message, we can take the hash of message and use it as an ID.
                let mut s = DefaultHasher::new();
                message.data.hash(&mut s);
                gossipsub::MessageId::from(s.finish().to_string())
            });
        #[cfg(debug_assertions)]
        {
            // This is set to aid debugging by not cluttering the log space
            builder.heartbeat_interval(Duration::from_secs(10));
        }
        let gossipsub_config = builder.build().expect("Could not build gossipsub config");

        // build a gossipsub network behaviour
        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(key.clone()),
            gossipsub_config,
        )
        .expect("Could not build gossipsub config");
        Behaviour {
            gossipsub,
            ping: ping::Behaviour::new(ping::Config::new()),
        }
    }
}

// async fn send_to_server(
//     write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
//     msg: tungstenite::Message,
// ) -> anyhow::Result<()> {
//     write.lock().await.send(msg).await?;
//     Ok(())
// }

// async fn send_pong(
//     write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
//     msg: Vec<u8>,
// ) -> anyhow::Result<()> {
//     send_to_server(write, tungstenite::Message::Pong(msg)).await?;
//     Ok(())
// }

// async fn send_c2s(
//     write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
//     message: C2s,
// ) -> anyhow::Result<()> {
//     let mut buf = Vec::new();
//     message.encode(&mut buf);
//     send_to_server(write, tungstenite::Message::Binary(buf)).await?;
//     Ok(())
// }

// async fn handle_server_message(
//     write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
//     msg: tungstenite::Message,
// ) -> anyhow::Result<()> {
//     match msg {
//         tungstenite::Message::Binary(msg) => {
//             let server_to_client::Msg { id, s2c } = server_to_client::Msg::decode(msg.as_ref())?;
//             let Some(s2c) = s2c else {
//                 return Err(anyhow::anyhow!("Server message is empty"));
//             };
//             handle_s2c(write, s2c).await?;
//         }
//         tungstenite::Message::Ping(ping) => {
//             log::debug!("{}", String::from_utf8_lossy(&ping));
//             send_pong(write, ping).await?;
//         }
//         tungstenite::Message::Close(close_frame) => {
//             log::info!("Session closed by server");
//             if let Some(close_frame) = close_frame {
//                 let CloseFrame { code, reason } = close_frame;
//                 log::debug!("Code: {code}");
//                 log::debug!("Reason: {reason}");
//             }
//             return Err(anyhow::anyhow!("Closed by server!"));
//         }
//         _ => {
//             log::error!("Unsupported Message type - Closing");
//             return Err(anyhow::anyhow!("Message type not supported!"));
//         }
//     }
//     Ok(())
// }

// async fn handle_s2c(
//     write: &Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tungstenite::Message>>,
//     msg: S2c,
// ) -> anyhow::Result<()> {
//     match msg {
//         S2c::NewGameEvent(e) => {
//             log::debug!("{e:?}")
//         }
//         S2c::NewGameResponse(r) => {
//             log::debug!("{r:?}")
//         }
//     }
//     Ok(())
// }
