use crate::db::users::NewUser;

#[derive(Debug, Clone)]
pub struct LichessClaims {
    pub id: String,
    pub username: String,
    pub email: String,
}

impl LichessClaims {
    pub fn into_db_user(self, user_name: String) -> NewUser {
        let Self { email, .. } = self;
        NewUser {
            user_name: user_name.clone(),
            display_name: user_name,
            email,
            locale: None,
            verified_email: false,
        }
    }
}
