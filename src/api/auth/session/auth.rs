use actix_web::{http::header::Header, web::Data, FromRequest};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use futures::future::LocalBoxFuture;
use uuid::Uuid;

use crate::app_error::AppError;

use super::{claims::Claims, config::Config};

pub struct Auth {
    pub user_id: Uuid,
}
impl Auth {
    pub fn is_user(&self, user_id: Uuid) -> bool {
        self.user_id == user_id
    }
    pub fn should_be_user(self, user_id: Uuid) -> Result<Auth, AppError> {
        if !self.is_user(user_id) {
            return Err(AppError::Unauthorized);
        }
        Ok(self)
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
