use std::io;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sea_orm::{DbErr, SqlErr, TransactionError};
use thiserror::Error;

use crate::api::p2p::P2pError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("database")]
    Db(DbErr),
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
    #[error("validate")]
    Validate(#[from] validator::ValidationErrors),
    #[error("p2p-{0}")]
    P2p(#[from] P2pError),
}

impl From<DbErr> for AppError {
    fn from(value: DbErr) -> Self {
        if let Some(SqlErr::UniqueConstraintViolation(message)) = value.sql_err() {
            if message.contains("users")
                && (message.contains("user_name") || message.contains("users_user_name_key"))
            {
                return AppError::UsernameAlreadyExists;
            }
        }
        AppError::Db(value)
    }
}

impl From<TransactionError<AppError>> for AppError {
    fn from(value: TransactionError<AppError>) -> Self {
        match value {
            TransactionError::Connection(err) => AppError::from(err),
            TransactionError::Transaction(err) => err,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Db(db_err) => match db_err {
                DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
                _ => match db_err.sql_err() {
                    Some(SqlErr::UniqueConstraintViolation(_))
                    | Some(SqlErr::ForeignKeyConstraintViolation(_)) => StatusCode::BAD_REQUEST,
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                },
            },
            AppError::Reqwest(_)
            | AppError::Unexpected
            | AppError::SerdeJson(_)
            | AppError::P2p(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Jwt(_) | AppError::OpenId | AppError::Unauthorized => {
                StatusCode::UNAUTHORIZED
            }
            AppError::AlreadyFriends
            | AppError::FriendRequestDoesntExist
            | AppError::FriendRequestExistsInOtherDirection
            | AppError::UsernameAlreadyExists
            | AppError::Validate(_) => StatusCode::BAD_REQUEST,
        };
        (
            status,
            Json(JsonError {
                error: self.to_string(),
                data: None,
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct JsonError {
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
}

impl From<AppError> for io::Error {
    fn from(value: AppError) -> Self {
        io::Error::other(value)
    }
}
