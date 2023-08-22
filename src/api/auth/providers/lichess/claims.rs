use crate::{
    api::auth::session::claims::Claims,
    db::users::{NewLichessUser, NewUser, NewUserWithId, UpdateLichessUser},
};

#[derive(Debug, Clone)]
pub struct LichessClaims {
    pub id: String,
    pub username: String,
    pub email: String,
}

impl LichessClaims {
    pub fn to_db_users(self, user_name: String) -> (NewLichessUser, NewUserWithId) {
        let user_id = uuid::Uuid::new_v4();
        let Self {
            email,
            id,
            username,
        } = self;
        let lichess_user = NewLichessUser {
            id,
            username: username.clone(),
            user_id,
        };
        let user = NewUserWithId {
            id: user_id,
            user_name: user_name.clone(),
            display_name: user_name,
            email,
            locale: None,
            verified_email: false,
        };
        (lichess_user, user)
    }
}

impl From<LichessClaims> for UpdateLichessUser {
    fn from(value: LichessClaims) -> Self {
        let LichessClaims { username, .. } = value;

        Self {
            username: Some(username.clone()),
        }
    }
}
