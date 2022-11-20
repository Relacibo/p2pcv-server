use std::cell::RefCell;

use actix_web::{
    web::{block, Data},
    FromRequest,
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use futures::future::LocalBoxFuture;

use crate::app_error::AppError;

pub type DbPool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
pub type DbConnection<'a> =
    bb8::PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

pub type DbExtractor<'a> = Data<DbConnection<'a>>;
