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
