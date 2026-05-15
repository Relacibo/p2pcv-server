use axum::Json;

use crate::error::AppError;

pub type EndpointResult<T> = Result<Json<T>, AppError>;
pub type AppResult<T> = Result<T, AppError>;
