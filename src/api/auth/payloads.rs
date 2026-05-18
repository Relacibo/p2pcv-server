use sea_orm::prelude::{DateTimeWithTimeZone, Uuid};

use crate::db::users::User;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserResponse {
    pub id: Uuid,
    pub user_name: String,
    pub display_name: String,
    pub email: String,
    pub locale: String,
    pub verified_email: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            user_name: u.user_name,
            display_name: u.display_name,
            email: u.email,
            locale: u.locale,
            verified_email: u.verified_email,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

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
        Self::Success(Box::new(LoginResponseSuccess {
            token,
            user: user.into(),
        }))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginResponseSuccess {
    token: String,
    user: UserResponse,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum OauthData {
    #[serde(rename_all = "camelCase")]
    Google { credential: String },
    #[serde(rename_all = "camelCase")]
    Lichess { code: String, code_verifier: String },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SigninPayload {
    pub oauth_data: OauthData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignupPayload {
    pub username: String,
    pub oauth_data: OauthData,
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
