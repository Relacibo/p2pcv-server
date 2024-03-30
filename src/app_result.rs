use actix_web::{web::Json, HttpResponse, Responder};

use crate::app_error::AppError;

pub type EndpointResult<T> = Result<Json<T>, AppError>;

pub type EndpointResultHttpResponse = Result<HttpResponse, AppError>;

pub type AppResult<T> = Result<T, AppError>;
