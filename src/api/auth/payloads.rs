use validator::ValidationError;

use crate::db::users::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case", tag = "result")]
pub enum LoginResponse {
    Success(Box<LoginResponseSuccess>),
    #[serde(rename_all = "camelCase")]
    NotRegistered {
        username_suggestion: String,
    },
}

impl LoginResponse {
    pub fn success(token: String, user: User) -> Self {
        Self::Success(Box::new(LoginResponseSuccess { token, user }))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponseSuccess {
    token: String,
    user: User,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum OauthData {
    #[serde(rename_all = "camelCase")]
    Google { credential: String },
    #[serde(rename_all = "camelCase")]
    Lichess {
        code: String,
        code_verifier: String,
        redirect_uri: Option<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigninPayload {
    pub oauth_data: OauthData,
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    if !username.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ValidationError::new("username must be alphanumeric"));
    }
    if let Some(first) = username.chars().next() {
        if first.is_ascii_digit() {
            return Err(ValidationError::new("username must not start with a digit"));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct SignupPayload {
    #[validate(custom(function = "validate_username"))]
    pub username: String,
    pub oauth_data: OauthData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GuestLoginPayload {
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderType {
    Google,
    Lichess,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectionsResponse {
    pub google: bool,
    pub lichess: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnlinkPayload {
    pub provider: ProviderType,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkPayload {
    pub oauth_data: OauthData,
}
