use chrono::Utc;
use redis::{AsyncCommands, FromRedisValue, RedisError, RedisWrite, ToRedisArgs, Value};
use redis_derive::{FromRedisValue, ToRedisArgs};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PeerConnection {
    id: Uuid,
    user_id: Uuid,
}

#[derive(Debug, Clone, FromRedisValue, ToRedisArgs)]
pub struct PeerConnectionHash {
    user_id: Uuid,
}

impl PeerConnection {
    pub async fn upsert(
        &self,
        conn: &mut redis::aio::MultiplexedConnection,
    ) -> Result<(), AppError> {
        let PeerConnection { id, user_id } = self;
        let key = format!("peer_connection:{id}");
        let _: u64 = conn.hset(&key, "user_id", user_id).await?;
        conn.expire(key, 60).await?;
        Ok(())
    }
    pub async fn list_for_user(
        conn: &mut redis::aio::MultiplexedConnection,
        user_id: Uuid,
    ) -> Result<Vec<Uuid>, AppError> {
        let result: Value = redis::cmd("FT.SEARCH")
            .arg("peer_connection:idx:user_id")
            .arg(user_id)
            .query_async(conn)
            .await?;
        let uuids = result
            .as_sequence()
            .ok_or(AppError::Unexpected)?
            .iter()
            .skip(1)
            .map(|v| PeerConnectionHash::from_redis_value(v).map(|v| v.user_id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(uuids)
    }
}
