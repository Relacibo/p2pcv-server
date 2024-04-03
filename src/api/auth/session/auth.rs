use actix_web::{http::header::Header, web::Data, FromRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use diesel_async::AsyncPgConnection;
use futures::future::LocalBoxFuture;
use uuid::Uuid;

use crate::{api::auth, app_error::AppError, db::users::User};

use super::{claims::Claims, config::Config};

pub struct Auth {
    pub user_id: Uuid,
}
impl Auth {
    pub fn is_user(&self, user_id: Uuid) -> bool {
        self.user_id == user_id
    }
    pub fn should_be_user(&self, user_id: Uuid) -> Result<(), AppError> {
        if !self.is_user(user_id) {
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }
    pub async fn should_be_friends_with(
        &self,
        conn: &mut AsyncPgConnection,
        other_user_id: Uuid,
    ) -> Result<(), AppError> {
        let are_friends = User::is_friends_with(conn, self.user_id, other_user_id).await?;
        if !are_friends {
            return Err(AppError::Unauthorized);
        }
        Ok(())
    }
}

impl FromRequest for Auth {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            let auth = Authorization::<Bearer>::parse(&req)?;
            let jwt = auth.as_ref().token();
            let Config {
                jwt_decoding_key,
                jwt_validation,
                ..
            } = req.app_data::<Data<Config>>().unwrap().as_ref();
            let claims =
                jsonwebtoken::decode::<Claims>(jwt, jwt_decoding_key, jwt_validation)?.claims;

            Ok(claims.into())
        })
    }
}
