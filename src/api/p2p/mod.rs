use std::{
    env,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    net::Ipv4Addr,
    str::FromStr,
    time::Duration,
};

use base64::{prelude::BASE64_STANDARD, Engine};
use futures::StreamExt;
use libp2p::{
    core::muxing::StreamMuxerBox,
    gossipsub,
    identity::Keypair,
    multiaddr::Protocol,
    ping,
    swarm::{behaviour, NetworkBehaviour, SwarmEvent},
    Multiaddr, PeerId, Swarm, SwarmBuilder, TransportError,
};
use libp2p_core::transport::Transport;
use libp2p_webrtc::tokio::Certificate;
use thiserror::Error;

pub async fn init() -> Result<(), P2pError> {
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
    private_key = private_key
        .into_iter()
        .rev()
        .take(32)
        .rev()
        .collect::<Vec<_>>();

    // Only the 32 last bytes are the actual key
    let keypair = Keypair::ed25519_from_bytes(private_key)
        .expect("P2P_PRIVATE_KEY_ED25519 is not a private key");

    let cert = env::var("P2P_TRANSPORT_CERT_PEM")
        .expect("P2P_TRANSPORT_CERT_PEM needed!")
        .replace("$", "\n");
    // webrtc::peer_connection::certificate::RTCCertificate::from_params(params)

    // https://github.com/libp2p/rust-libp2p/blob/a2a281609a0a64b211f7917aa856924983b63200/examples/browser-webrtc/src/main.rs#L25
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

    loop {
        tokio::select! {
            swarm_event = swarm.next() => {
                if let Some(swarm_event) = swarm_event {
                    if let Err(err) = handle_swarm_event(&swarm, swarm_event).await {
                        log::error!("{err}");
                    }
                }
            },
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }

    Ok(())
}

async fn handle_swarm_event(
    swarm: &Swarm<Behaviour>,
    swarm_event: SwarmEvent<BehaviourEvent>,
) -> Result<(), P2pError> {
    log::debug!("{:?}", swarm_event);
    match swarm_event {
        SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(event)) => {
            handle_gossipsub_event(swarm, event).await?
        }
        _ => (),
    }
    Ok(())
}

async fn handle_gossipsub_event(
    swarm: &Swarm<Behaviour>,
    event: gossipsub::Event,
) -> Result<(), P2pError> {
    let Behaviour { gossipsub, .. } = swarm.behaviour();
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
            ping: Default::default(),
        }
    }
}

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("init-p2p")]
    InitP2p,
    #[error("transport")]
    Transport(#[from] TransportError<io::Error>),
}

impl From<P2pError> for io::Error {
    fn from(value: P2pError) -> Self {
        io::Error::new(io::ErrorKind::Other, value)
    }
}
