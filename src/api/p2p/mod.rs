use std::{
    env,
    hash::{DefaultHasher, Hash, Hasher},
    io,
    time::Duration,
};

use libp2p::{gossipsub, identity::Keypair, swarm::NetworkBehaviour, SwarmBuilder};
use thiserror::Error;

use crate::error::AppError;

pub async fn init() -> Result<(), P2pError> {
    let timeout_secs = env::var("P2P_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(60);
    let host = env::var("P2P_HOST").expect("P2P_HOST needed!");
    let port = env::var("P2P_PORT").expect("P2P_PORT needed!");
    let peer_id = env::var("P2P_PEER_ID").expect("P2P_PEER_ID needed!");
    let mut swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_quic()
        .with_behaviour(Behaviour::create)
        .map_err(|_| P2pError::InitP2p)?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(Duration::from_secs(timeout_secs))
        })
        .build();
    swarm.listen_on(
        format!("/ip4/{host}:{port}/udp/0/quic-v1")
            .parse()
            .expect("Could not parse p2p address!"),
    )?;
    Ok(())
}

// We create a custom network behaviour that combines Gossipsub and Mdns.
#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
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
        Behaviour { gossipsub }
    }
}

#[derive(Debug, Error)]
pub enum P2pError {
    #[error("init-p2p")]
    InitP2p,
}
