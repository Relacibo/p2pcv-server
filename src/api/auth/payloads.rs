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
    Google { credentials: String },
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
