use std::sync::Arc;

use actix_web::{web::Data, HttpRequest};
use async_trait::async_trait;
use diesel_async::AsyncPgConnection;

use crate::{
    api::auth::providers::provider::{Provider, ProviderError},
    app_result::AppResult,
    db::{
        db_conn::DbPool,
        lichess::{LichessAccessToken, NewLichessAccessToken},
        users::{UpdateLichessUser, User},
    },
    error::AppError,
};

use super::{claims::LichessClaims, config::Config};

#[derive(Clone, Debug)]
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
        let pool = req.app_data::<Data<DbPool>>().unwrap().clone().into_inner();
        let reqwest = req
            .app_data::<Data<reqwest::Client>>()
            .unwrap()
            .clone()
            .into_inner();
        let mut db = pool.get().await?;

        let claims =
            request_lichess_claims(&reqwest, &config, code, code_verifier, &mut db).await?;
        ;

        Ok(Self { claims })
    }
}

async fn request_lichess_claims(
    reqwest: &reqwest::Client,
    config: &Config,
    code: String,
    code_verifier: String,
    conn: &mut AsyncPgConnection,
) -> AppResult<LichessClaims> {
    let Config {
        api_uri,
        client_id,
        redirect_uri,
        token_endpoint_path,
        email_endpoint_path,
        account_endpoint_path,
        ..
    } = config;

    let access_token = LichessAccessToken::get(conn, code_verifier.clone()).await?;

    let access_token = if let Some(LichessAccessToken { access_token, .. }) = access_token {
        access_token
    } else {
        let token_endpoint = format!("{api_uri}{token_endpoint_path}");

        let token_request = LichessTokenRequest {
            grant_type: "authorization_code".to_string(),
            code,
            code_verifier: code_verifier.clone(),
            redirect_uri: redirect_uri.clone(),
            client_id: client_id.clone(),
        };

        let LichessTokenResponse {
            access_token,
            expires_in,
            ..
        } = reqwest
            .post(token_endpoint)
            .form(&token_request)
            .send()
            .await?
            .json()
            .await?;

        let t = NewLichessAccessToken {
            id: code_verifier.clone(),
            access_token: access_token.clone(),
            expires: expires_in as i64,
        };
        LichessAccessToken::insert(conn, t).await?;
        access_token
    };

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
        .bearer_auth(access_token.clone())
        .send()
        .await?
        .json()
        .await?;
    Ok(LichessClaims {
        id,
        username,
        email,
    })
}

#[async_trait]
impl Provider for LichessProvider {
    async fn get_updated_user(&self, conn: &mut AsyncPgConnection) -> Result<User, ProviderError> {
        let Self { claims } = self;
        let LichessClaims { id, username, .. } = claims;
        let update_lichess_user: UpdateLichessUser = claims.clone().into();
        let user = User::get_with_lichess_id(conn, id).await?.ok_or_else(|| {
            ProviderError::UserNotFound {
                user_name: username.clone(),
            }
        })?;

        User::update_lichess_user(conn, id, update_lichess_user).await?;
        Ok(user)
    }
    async fn insert_user(
        &self,
        conn: &mut AsyncPgConnection,
        username: &str,
    ) -> Result<User, ProviderError> {
        let Self { claims } = self;
        let (new_lichess_user, new_user) =
            claims.clone().to_db_users(username.to_string());
        let insert_result = User::insert_lichess_user(conn, new_user, new_lichess_user).await;
        let (_, user) = match insert_result {
            Ok(user) => user,
            Err(AppError::UsernameAlreadyExists) => Err(ProviderError::UserAlreadyExists {
                user_name: username.to_string(),
            })?,
            res => res?,
        };
        Ok(user)
    }
}
