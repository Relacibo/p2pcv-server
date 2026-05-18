use async_trait::async_trait;
use sea_orm::DatabaseConnection;

use crate::{
    AppState,
    api::auth::providers::provider::{Provider, ProviderError},
    app_result::AppResult,
    db::users::User,
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
    pub async fn new(state: &AppState, code: String, code_verifier: String) -> AppResult<Self> {
        let claims = request_lichess_claims(
            &state.reqwest_client,
            &state.lichess_config,
            code,
            code_verifier,
        )
        .await?;
        Ok(Self { claims })
    }
}

async fn request_lichess_claims(
    reqwest: &reqwest::Client,
    config: &Config,
    code: String,
    code_verifier: String,
) -> AppResult<LichessClaims> {
    let Config {
        api_uri,
        client_id,
        redirect_uri,
        token_endpoint_path,
        email_endpoint_path,
        account_endpoint_path,
    } = config;

    let token_endpoint = format!("{api_uri}{token_endpoint_path}");
    let token_request = LichessTokenRequest {
        grant_type: "authorization_code".to_string(),
        code,
        code_verifier,
        redirect_uri: redirect_uri.clone(),
        client_id: client_id.clone(),
    };

    let LichessTokenResponse {
        access_token,
        ..
    } = reqwest
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

    Ok(LichessClaims {
        id,
        username,
        email,
    })
}

#[async_trait]
impl Provider for LichessProvider {
    async fn get_updated_user(&self, db: &DatabaseConnection) -> Result<User, ProviderError> {
        let LichessClaims { id, username, .. } = &self.claims;
        let user = User::get_with_lichess_id(db, id).await?.ok_or_else(|| {
            ProviderError::UserNotFound {
                user_name: username.clone(),
            }
        })?;
        User::update_provider_display_name(db, user.id, "lichess", username).await?;
        Ok(user)
    }

    async fn insert_user(
        &self,
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<User, ProviderError> {
        let LichessClaims {
            id,
            username: lichess_username,
            ..
        } = &self.claims;
        let new_user = self.claims.clone().to_db_user(username.to_string());
        let insert_result =
            User::insert_with_provider(db, new_user, "lichess", id, Some(lichess_username)).await;
        let user = match insert_result {
            Ok(user) => user,
            Err(AppError::UsernameAlreadyExists) => Err(ProviderError::UserAlreadyExists {
                user_name: username.to_string(),
            })?,
            Err(err) => Err(err)?,
        };
        Ok(user)
    }

    async fn link_to_user(
        &self,
        db: &DatabaseConnection,
        user_id: uuid::Uuid,
    ) -> Result<(), ProviderError> {
        let LichessClaims { id, username, .. } = &self.claims;
        User::link_lichess(db, user_id, id, username).await?;
        Ok(())
    }
}
