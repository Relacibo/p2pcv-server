pub mod sc_requests {
    include!(concat!(env!("OUT_DIR"), "/org.ggchess.server_client.requests.rs"));
}

pub mod sc_responses {
    include!(concat!(env!("OUT_DIR"), "/org.ggchess.server_client.responses.rs"));
}

pub mod p2p_messages {
    include!(concat!(env!("OUT_DIR"), "/org.ggchess.p2p.messages.rs"));
}
