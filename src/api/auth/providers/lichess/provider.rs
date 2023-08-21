use std::sync::Arc;

use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use diesel_async::AsyncPgConnection;

use crate::{
    api::auth::{
        providers::{map_to_provider_error, Provider, ProviderError},
        public_key_storage::KeyStore,
    },
    app_error::AppError,
    app_result::AppResult,
    db::users::{NewUser, User},
};

use super::{claims::LichessClaims, config::Config};

pub struct LichessProvider {
    pub claims: LichessClaims,
}

#[derive(Debug, Serialize)]
struct LichessTokenRequest {
    grant_type: String,
    code: String,
    code_verifier: String,
    redirect_uri: String,
    client_id: String,
}

#[derive(Debug, Deserialize)]
struct LichessTokenResponse {
    token_type: String,
    access_token: String,
    expires_in: usize,
}

#[derive(Debug, Deserialize)]
struct LichessEmailResponse {
    email: String,
}

#[derive(Debug, Deserialize)]
struct LichessAccountResponse {
    id: String,
    username: String,
}

impl LichessProvider {
    pub async fn new(req: &HttpRequest, code: String, code_verifier: String) -> AppResult<Self> {
        let config = req.app_data::<Data<Config>>().unwrap().clone().into_inner();
        let reqwest = req
            .app_data::<Data<reqwest::Client>>()
            .unwrap()
            .clone()
            .into_inner();
        let Config {
            api_uri,
            client_id,
            redirect_uri,
            token_endpoint_path,
            email_endpoint_path,
            account_endpoint_path,
            ..
        } = config.as_ref();
        let token_endpoint = format!("{api_uri}{token_endpoint_path}");

        let token_request = LichessTokenRequest {
            grant_type: "authorization_code".to_string(),
            code,
            code_verifier,
            redirect_uri: redirect_uri.clone(),
            client_id: client_id.clone(),
        };
        let LichessTokenResponse { access_token, .. } = reqwest
            .post(token_endpoint)
            .form(&token_request)
            .send()
            .await?
            .json()
            .await?;

        let endpoint_path = format!("{api_uri}{account_endpoint_path}");
        let LichessAccountResponse { id, username } = reqwest
            .get(endpoint_path)
            .bearer_auth(access_token.clone())
            .send()
            .await?
            .json()
            .await?;

        let email_endpoint = format!("{api_uri}{email_endpoint_path}");
        let LichessEmailResponse { email } = reqwest
            .get(email_endpoint)
            .bearer_auth(access_token)
            .send()
            .await?
            .json()
            .await?;
        let claims = LichessClaims {
            id,
            username,
            email,
        };

        Ok(Self { claims })
    }
}

#[async_trait]
impl Provider for LichessProvider {
    async fn try_get_user(&self, mut conn: &mut AsyncPgConnection) -> Result<User, ProviderError> {
        let Self { claims } = self;
        let LichessClaims {
            id,
            username,
            email,
        } = claims;

        let user_result = User::get_with_google_id(&mut conn, &sub)
            .await
            .map_err(map_to_provider_error)?;
        let user_result = if let Some(user) = user_result {
            User::update_google_user(&mut conn, user.id, claims.clone().into())
                .await
                .map_err(map_to_provider_error)?
        } else {
            user_result
        };
        user_result.ok_or_else(|| ProviderError::UserNotFound {
            user_name: name.clone(),
        })
    }
    async fn try_insert_user(
        &self,
        mut conn: &mut AsyncPgConnection,
    ) -> Result<User, ProviderError> {
        let Self {
            credential,
            keystore,
            config,
        } = self;
        let ref claims @ GoogleClaims {
            ref sub, ref name, ..
        } = extract_google_claims(&config, &keystore, credential).await?;
        let new_user: NewUser = GoogleClaims::to_database_entry(claims.clone(), name.clone());
        let insert_result = User::insert_with_google_id(&mut conn, new_user, &sub).await;
        let user = match insert_result {
            Ok(user) => user,
            Err(AppError::UsernameAlreadyExists) => Err(ProviderError::UserAlreadyExists {
                user_name: name.clone(),
            })?,
            res => res?,
        };
        Ok(user)
    }
}
