use std::ops::{Deref, DerefMut};

use actix_web::{web::Data, FromRequest};
use bb8::PooledConnection;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use futures::future::LocalBoxFuture;

use crate::error::AppError;

use super::db_conn::{DbConnection, DbPool};

pub struct DbConn(pub DbConnection<'static>);

impl FromRequest for DbConn {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let db_data = req
            .app_data::<Data<DbPool>>()
            .expect("Postgres Datapool not in data!");
        let db_arc = db_data.clone().into_inner();
        Box::pin(async move {
            let db = db_arc.get_owned().await?;
            Ok(DbConn(db))
        })
    }
}

impl Deref for DbConn {
    type Target = AsyncPgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DbConn {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
