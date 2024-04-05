use actix_web::{
    web::{Data},
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

pub type DbPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type DbConnection<'a> =
    bb8::PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;
