use actix_web::{error::ParseError, http::StatusCode, HttpResponseBuilder};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("unknown")]
    ActixBlocking(#[from] actix_web::error::BlockingError),
    #[error("database")]
    Diesel(diesel::result::Error),
    #[error("unknown")]
    Bb8,
    #[error("authentication-failed")]
    JwtParse(#[from] ParseError),
    #[error("authentication-failed")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("authentication-failed")]
    OpenId,
    #[error("unknown")]
    Reqwest(#[from] reqwest::Error),
    #[error("unknown")]
    SerdeJson(#[from] serde_json::error::Error),
    #[error("unknown")]
    Unexpected,
    #[error("unauthorized")]
    Unauthorized,
    #[error("already-friends")]
    AlreadyFriends,
    #[error("friend-request-doesnt-exist")]
    FriendRequestDoesntExist,
    #[error("friend-request-exists-in-other-direction")]
    FriendRequestExistsInOtherDirection,
    #[error("username-already-exists")]
    UsernameAlreadyExists,
}

impl<E> From<bb8::RunError<E>> for AppError {
    fn from(_value: bb8::RunError<E>) -> Self {
        AppError::Bb8
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(value: diesel::result::Error) -> Self {
        use diesel::result::DatabaseErrorKind;
        use diesel::result::Error::*;

        let (kind, table_name, column_name) = if let DatabaseError(ref kind, ref err) = value {
            let column_name = err.column_name().map(|s| s.to_string());
            let table_name = err.table_name().map(|s| s.to_string());
            (Some(kind), table_name, column_name)
        } else {
            (None, None, None)
        };
        #[allow(clippy::single_match)]
        match kind {
            Some(DatabaseErrorKind::UniqueViolation) => {
                match (table_name.as_deref(), column_name.as_deref()) {
                    (Some("users"), Some("user_name")) => return AppError::UsernameAlreadyExists,
                    _ => (),
                }
            }
            _ => (),
        }
        AppError::Diesel(value)
    }
}

impl actix_web::ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        use diesel::result::DatabaseErrorKind::{ForeignKeyViolation, UniqueViolation};
        use diesel::result::Error::{DatabaseError, NotFound};
        use AppError::*;
        match self {
            Diesel(diesel_err) => match diesel_err {
                NotFound => StatusCode::NOT_FOUND,
                DatabaseError(UniqueViolation | ForeignKeyViolation, _) => StatusCode::BAD_REQUEST,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            },
            ActixBlocking(_) | Bb8 | Reqwest(_) | Unexpected | SerdeJson(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            JwtParse(_) | Jwt(_) | OpenId | Unauthorized => StatusCode::UNAUTHORIZED,
            AlreadyFriends
            | FriendRequestDoesntExist
            | FriendRequestExistsInOtherDirection
            | UsernameAlreadyExists => StatusCode::BAD_REQUEST,
        }
    }
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponseBuilder::new(self.status_code()).json(JsonError {
            error: &self.to_string(),
            data: None,
        })
    }
}
#[derive(Serialize)]
struct JsonError<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<&'a str>,
}
