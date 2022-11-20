use actix_web::web::Json;

use crate::app_error::AppError;

pub type EndpointResult<T> = Result<Json<T>, AppError>;
