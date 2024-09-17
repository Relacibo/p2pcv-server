pub mod requests {
    include!(concat!(env!("OUT_DIR"), "/org.ggchess.proto.requests.rs"));
}

pub mod responses {
    include!(concat!(env!("OUT_DIR"), "/org.ggchess.proto.responses.rs"));
}
