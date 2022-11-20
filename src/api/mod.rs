use actix_web::{
    web::{block, Data},
    FromRequest,
};
use diesel::{r2d2::ConnectionManager, PgConnection};
use futures::future::LocalBoxFuture;

use crate::DbPool;

use self::app_error::AppError;

pub mod auth;
pub mod app_error;
pub mod users;

pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;
pub struct DbConn(DbConnection);

impl From<DbConnection> for DbConn {
    fn from(conn: DbConnection) -> Self {
        DbConn(conn)
    }
}

impl FromRequest for DbConn {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let pool = req.app_data::<Data<DbPool>>().unwrap().clone();
        Box::pin(async move { Ok(pool.get()?.into()) })
    }
}
