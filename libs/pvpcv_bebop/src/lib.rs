//! Bebop-format binary encoding for the p2p signaling protocol.
//!
//! Types are generated from schemas in `schemas/` via `bebopc` (see `build.rs`).
//! The `bebop-owned-all` feature produces owned variants (no `'de` lifetimes).

mod generated;

pub use bebop::Guid;
pub use generated::c2s::owned::C2SMsg as C2sMsg;
pub use generated::s2c::owned::S2CMsg as S2cMsg;

use std::io;

// ── Guid ↔ Uuid conversions ──────────────────────────────────────────────────

/// Extension methods on `bebop::Guid` for working with `uuid::Uuid`.
pub trait GuidExt: Sized {
    fn to_uuid(self) -> uuid::Uuid;
    fn is_nil(self) -> bool;
}

impl GuidExt for bebop::Guid {
    fn to_uuid(self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(self.to_be_bytes())
    }
    fn is_nil(self) -> bool {
        self.to_uuid().is_nil()
    }
}

/// Extension methods on `uuid::Uuid` for conversion to `bebop::Guid`.
pub trait UuidExt {
    fn to_guid(self) -> bebop::Guid;
}

impl UuidExt for uuid::Uuid {
    fn to_guid(self) -> bebop::Guid {
        bebop::Guid::from_be_bytes(*self.as_bytes())
    }
}

// ── serialize / deserialize helpers ─────────────────────────────────────────

fn to_vec(record: &impl bebop::Record<'static>) -> Vec<u8> {
    let mut v = Vec::new();
    record.serialize(&mut v).expect("bebop serialization failed");
    v
}

fn from_bytes<T: for<'r> bebop::Record<'r>>(raw: &[u8]) -> io::Result<T> {
    T::deserialize(raw).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
}

// ── Inherent serialize / deserialize on message types ───────────────────────

impl generated::c2s::owned::C2SMsg {
    pub fn serialize(&self) -> Vec<u8> {
        to_vec(self)
    }
    pub fn deserialize(raw: &[u8]) -> io::Result<Self> {
        from_bytes(raw)
    }
}

impl generated::s2c::owned::S2CMsg {
    pub fn serialize(&self) -> Vec<u8> {
        to_vec(self)
    }
    pub fn deserialize(raw: &[u8]) -> io::Result<Self> {
        from_bytes(raw)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use bebop::Guid;

    use super::{C2sMsg, S2cMsg, UuidExt};

    fn nil_guid() -> Guid {
        uuid::Uuid::nil().to_guid()
    }

    fn some_guid() -> Guid {
        uuid::Uuid::new_v4().to_guid()
    }

    // ── C2sMsg round-trips ────────────────────────────────────────────────────

    #[test]
    fn c2s_register_peer_round_trip() {
        let msg = C2sMsg::RegisterPeer { auth_token: "tok123".into() };
        let bytes = msg.serialize();
        assert_eq!(C2sMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn c2s_get_friend_peer_id_round_trip() {
        let msg = C2sMsg::GetFriendPeerId { friend_user_id: some_guid() };
        let bytes = msg.serialize();
        assert_eq!(C2sMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn c2s_new_game_round_trip() {
        let msg = C2sMsg::NewGame {
            receiver_user_id: some_guid(),
            variant_id: some_guid(),
            variant_version: "1.0.0".into(),
        };
        let bytes = msg.serialize();
        assert_eq!(C2sMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn c2s_new_game_answer_accepted_round_trip() {
        let msg = C2sMsg::NewGameAnswer { accepted: true };
        let bytes = msg.serialize();
        assert_eq!(C2sMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn c2s_new_game_answer_rejected_round_trip() {
        let msg = C2sMsg::NewGameAnswer { accepted: false };
        let bytes = msg.serialize();
        assert_eq!(C2sMsg::deserialize(&bytes).unwrap(), msg);
    }

    // ── S2cMsg round-trips ────────────────────────────────────────────────────

    #[test]
    fn s2c_register_peer_response_success_round_trip() {
        let msg = S2cMsg::RegisterPeerResponse { success: true };
        let bytes = msg.serialize();
        assert_eq!(S2cMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn s2c_register_peer_response_failure_round_trip() {
        let msg = S2cMsg::RegisterPeerResponse { success: false };
        let bytes = msg.serialize();
        assert_eq!(S2cMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn s2c_get_friend_peer_id_response_with_peer_round_trip() {
        let msg = S2cMsg::GetFriendPeerIdResponse { peer_id: Some("12D3K".into()) };
        let bytes = msg.serialize();
        assert_eq!(S2cMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn s2c_get_friend_peer_id_response_no_peer_round_trip() {
        let msg = S2cMsg::GetFriendPeerIdResponse { peer_id: None };
        let bytes = msg.serialize();
        assert_eq!(S2cMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn s2c_new_game_response_accepted_round_trip() {
        let msg = S2cMsg::NewGameResponse {
            accepted: Some(true),
            receiver_peer_id: Some("peer-abc".into()),
        };
        let bytes = msg.serialize();
        assert_eq!(S2cMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn s2c_new_game_response_declined_round_trip() {
        let msg = S2cMsg::NewGameResponse { accepted: Some(false), receiver_peer_id: None };
        let bytes = msg.serialize();
        assert_eq!(S2cMsg::deserialize(&bytes).unwrap(), msg);
    }

    #[test]
    fn s2c_new_game_event_round_trip() {
        let msg = S2cMsg::NewGameEvent {
            sender_user_id: some_guid(),
            sender_user_name: "alice".into(),
            variant_id: some_guid(),
            variant_version: "2.1.0".into(),
            timeout_secs: 60,
        };
        let bytes = msg.serialize();
        assert_eq!(S2cMsg::deserialize(&bytes).unwrap(), msg);
    }

    // ── Guid ↔ Uuid conversions ───────────────────────────────────────────────

    #[test]
    fn guid_uuid_round_trip() {
        use super::GuidExt;
        let original = uuid::Uuid::new_v4();
        let guid = original.to_guid();
        let back = guid.to_uuid();
        assert_eq!(original, back);
    }

    #[test]
    fn nil_guid_is_nil() {
        use super::GuidExt;
        assert!(nil_guid().is_nil());
    }

    #[test]
    fn non_nil_guid_is_not_nil() {
        use super::GuidExt;
        assert!(!some_guid().is_nil());
    }

    #[test]
    fn c2s_deserialize_empty_bytes_returns_error() {
        assert!(C2sMsg::deserialize(&[]).is_err());
    }

    #[test]
    fn s2c_deserialize_empty_bytes_returns_error() {
        assert!(S2cMsg::deserialize(&[]).is_err());
    }

    #[test]
    fn s2c_deserialize_garbage_panics_or_errors() {
        // Bebop panics on some malformed inputs rather than returning Err —
        // both outcomes (Err or panic) indicate the data was rejected.
        let result = std::panic::catch_unwind(|| S2cMsg::deserialize(&[0xDE, 0xAD, 0xBE, 0xEF]));
        match result {
            Ok(inner) => assert!(inner.is_err(), "expected Err on garbage input"),
            Err(_) => {} // panic is acceptable
        }
    }
}
