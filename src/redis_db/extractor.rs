use std::ops::Deref;

use actix_web::{web::Data, FromRequest};
use futures::future::LocalBoxFuture;
use redis::aio::MultiplexedConnection;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct RedisClient(pub MultiplexedConnection);

impl FromRequest for RedisClient {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let redis_data = req
            .app_data::<Data<redis::Client>>()
            .expect("Redis-Client not in data!");
        let redis_arc = redis_data.into_inner().clone();
        Box::pin(async move {
            let conn = redis_arc.get_multiplexed_async_connection().await?;
            Ok(RedisClient(conn))
        })
    }
}

impl Deref for RedisClient {
    type Target = MultiplexedConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
