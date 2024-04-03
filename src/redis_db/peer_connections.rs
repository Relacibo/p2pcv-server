use chrono::Utc;
use redis::AsyncCommands;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PeerConnection {
    id: Uuid,
    user_id: Uuid,
}

impl PeerConnection {
    pub async fn upsert(&self, conn: &redis::aio::MultiplexedConnection) {
        let PeerConnection { id, user_id } = self;
        let now = Utc::now();
        conn.set(format!("{}_user_id", id), user_id);
        conn.set(format!("{}_updated_at", id), now);
    }
    pub async fn list_for_user(conn: &redis::aio::MultiplexedConnection, user_id: Uuid) {
        let minimum_last_ping = Utc::now() - Duration::minutes(1.5f32);
        conn.get("*_user_id")
        conn.set(format!("{}_user_id", id), user_id);
        conn.set(format!("{}_updated_at", id), now);
    }
}
