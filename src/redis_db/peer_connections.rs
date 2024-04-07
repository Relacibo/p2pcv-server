use chrono::{Duration, TimeZone, Utc};
use rand::{thread_rng, Rng};
use redis::AsyncCommands;
use redis_derive::{FromRedisValue, ToRedisArgs};
use uuid::Uuid;

use crate::error::AppError;

static CLEANUP_PROBABILITY: f64 = 0.002;

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
    // pub async fn upsert(
    //     &self,
    //     conn: &mut redis::aio::MultiplexedConnection,
    // ) -> Result<(), AppError> {
    //     let PeerConnection { id, user_id } = self;
    //     let now = Utc::now();
    //     let score = now.timestamp();
    //     let key = format!("users:peer_connections:{user_id}");
    //     let _: u64 = conn.zadd(&key, id, score).await?;
    //     // Only remove stale entries approx. every 500th time
    //     if thread_rng().gen_bool(CLEANUP_PROBABILITY) {
    //         let dropoff_score = (now - Duration::minutes(1)).timestamp();
    //         let _: u64 = conn.zrembyscore(key, -1, dropoff_score - 1).await?;
    //     }
    //     Ok(())
    // }
    // pub async fn list_for_user(
    //     conn: &mut redis::aio::MultiplexedConnection,
    //     user_id: Uuid,
    // ) -> Result<Vec<Uuid>, AppError> {
    //     let key = format!("users:peer_connections:{user_id}");
    //     let dropoff_score = (Utc::now() - Duration::minutes(1)).timestamp() as isize;
    //     let result: Vec<String> = conn.zrange(key, dropoff_score, -1).await?;
    //     println!("{result:?}");
    //     todo!();
    // }
}
