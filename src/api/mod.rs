use actix_web::{
    web::{block, Data},
    FromRequest,
};
use futures::future::LocalBoxFuture;

use crate::{DbConnection, DbPool};

use self::db_error::DbError;

pub mod auth;
pub mod db_error;
pub mod users;

struct DbConn(DbConnection);

impl From<DbConnection> for DbConn {
    fn from(conn: DbConnection) -> Self {
        DbConn(conn)
    }
}

impl FromRequest for DbConn {
    type Error = DbError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        Box::pin(async move {
            let pool = Data::<DbPool>::from_request(req, payload).await.unwrap();
            let connection = pool.get()?;
            let res = block(move || connection).await?;
            Ok(res.into())
        })
    }
}
