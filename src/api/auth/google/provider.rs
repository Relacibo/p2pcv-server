use std::sync::Arc;

use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use diesel_async::AsyncPgConnection;

use crate::{
    api::auth::{login::provider::Provider, public_key_storage::KeyStore},
    app_error::AppError,
    db::users::User,
};

use super::config::Config;

pub struct GoogleProvider {
    pub credentials: String,
    pub keystore: Arc<KeyStore>,
    pub config: Config,
}

impl GoogleProvider {
    pub fn new(req: &HttpRequest, credentials: String) -> Self {
        let keystore = req
            .app_data::<Data<KeyStore>>()
            .unwrap()
            .into_inner()
            .clone();
        let config = *req.app_data::<Data<Config>>().unwrap().into_inner().clone();
        Self {
            credentials,
            keystore,
            config,
        }
    }
}

#[async_trait]
impl Provider for GoogleProvider {
    async fn try_get_user(&self, conn: &mut AsyncPgConnection) -> Result<User, AppError> {
        Ok(todo!())
    }
    async fn try_insert_user(&self, conn: &mut AsyncPgConnection) -> Result<User, AppError> {
        Ok(todo!())
    }
}

// async fn signin(
//     config: Data<Config>,
//     key_store: Data<KeyStore>,
//     pool: Data<DbPool>,
//     payload: Json<SigninPayload>,
// ) -> EndpointResult<LoginResponse> {
//     let mut db = pool.get().await?;
//     let SigninPayload { credential } = payload.into_inner();

//     let claims = extract_google_claims(&config, &key_store, &credential).await?;
//     let sub = claims.sub.clone();
//     let name = claims.name.clone();

//     let user_result = User::get_with_google_id(&mut db, &sub).await;
//     let user = match user_result {
//         Ok(user) => {
//             User::update_google_user(&mut db, user.id, claims.into()).await?;
//             user
//         }
//         Err(NotFound) => {
//             let username_suggestion = suggest_username(&db, &name).await?;
//             return Ok(Json(LoginResponse::NotRegistered {
//                 username_suggestion,
//             }));
//         }
//         err => err?,
//     };
//     let token = generate_login_token(&jwt_config, user.id)?;
//     let res = LoginResponse::success(token, user);
//     Ok(Json(res))
// }

// async fn extract_google_claims(
//     config: &Config,
//     key_store: &KeyStore,
//     credential: &str,
// ) -> Result<GoogleClaims, AppError> {
//     /* https://developers.google.com/identity/gsi/web/guides/verify-google-id-token?hl=en */
//     // Find out kid to use
//     let header = decode_header(credential)?;
//     let kid = header.kid.ok_or(AppError::OpenId)?;

//     let key = key_store.get_key(&kid).await?;

//     let PublicKey { n, e, .. } = &key;
//     let decoding_key = DecodingKey::from_rsa_raw_components(n, e);
//     let mut validation = Validation::new(Algorithm::RS256);
//     validation.set_audience(&[config.client_id.clone()]);
//     validation.set_issuer(&config.issuer);
//     let ticket = decode::<GoogleClaims>(credential, &decoding_key, &validation)?;
//     Ok(ticket.claims)
// }

// async fn signup(
//     config: Data<Config>,
//     jwt_config: Data<session::Config>,
//     key_store: Data<KeyStore>,
//     pool: Data<DbPool>,
//     payload: Json<SignupPayload>,
// ) -> EndpointResult<LoginResponse> {
//     let mut db = pool.get().await?;
//     let SignupPayload {
//         username,
//         credential,
//     } = &*payload;

//     let claims = extract_google_claims(&config, &key_store, credential).await?;
//     let sub = claims.sub.clone();

//     let new_user: NewUser = user_from_google_claims_and_username(claims, username.clone());
//     let insert_result = User::insert_with_google_id(&mut db, new_user, &sub).await;
//     let user = match insert_result {
//         Err(AppError::Diesel(DatabaseError(DatabaseErrorKind::UniqueViolation, a)))
//             if a.table_name() == Some("users") =>
//         {
//             let username_suggestion = suggest_username(&db, username).await?;
//             let res = LoginResponse::NotRegistered {
//                 username_suggestion,
//             };
//             return Ok(Json(res));
//         }
//         res => res?,
//     };
//     let token = generate_login_token(&jwt_config, user.id)?;
//     let res = LoginResponse::success(token, user);
//     Ok(Json(res))
// }
