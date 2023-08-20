use crate::db::users::{UpdateUserGoogle, NewUser};

#[derive(Clone, Debug, Deserialize)]
struct GoogleClaims {
    aud: String,
    exp: usize,
    iat: usize,
    iss: String,
    nbf: usize,
    sub: String,
    name: String,
    nick_name: Option<String>,
    given_name: Option<String>,
    middle_name: Option<String>,
    family_name: Option<String>,
    email: String,
    locale: Option<String>,
    #[serde(default)]
    verified_email: bool,
    picture: Option<String>,
}

impl GoogleClaims {
    fn to_database_entry(self, user_name: String) -> NewUser {
        let Self {
            email,
            locale,
            verified_email,
            ..
        } = self;
        NewUser {
            user_name: user_name.clone(),
            display_name: user_name,
            email,
            locale,
            verified_email,
        }
    }
}

impl From<GoogleClaims> for UpdateUserGoogle {
    fn from(val: GoogleClaims) -> Self {
        let GoogleClaims {
            email,
            locale,
            verified_email,
            ..
        } = val;
        UpdateUserGoogle {
            email: Some(email),
            locale,
            verified_email: Some(verified_email),
        }
    }
}
