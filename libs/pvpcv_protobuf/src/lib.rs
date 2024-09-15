pub mod client_to_server {
    include!(concat!(
        env!("OUT_DIR"),
        "/org.ggchess.proto.client_to_server.rs"
    ));
}

pub mod server_to_client {
    include!(concat!(
        env!("OUT_DIR"),
        "/org.ggchess.proto.server_to_client.rs"
    ));
}
