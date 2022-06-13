use core::fmt;
use std::error;

use actix_web::{http::StatusCode, HttpResponseBuilder};

#[derive(Debug)]
pub enum DbError {
    ActixBlocking(actix_web::error::BlockingError),
    Diesel(diesel::result::Error),
    R2d2(r2d2::Error),
}

impl actix_web::ResponseError for DbError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        use diesel::result::Error::{
            DeserializationError, NotFound, QueryBuilderError, SerializationError,
        };
        use DbError::*;
        match self {
            Diesel(diesel_err) => match diesel_err {
                NotFound => {
                    from_error(StatusCode::NOT_FOUND, diesel_err.to_string().as_str(), None)
                }
                QueryBuilderError(_) | SerializationError(_) | DeserializationError(_) => {
                    from_error(
                        StatusCode::BAD_REQUEST,
                        diesel_err.to_string().as_str(),
                        None,
                    )
                }
                _ => server_error(),
            },
            _ => server_error(),
        }
    }
}

fn server_error() -> actix_web::HttpResponse<actix_web::body::BoxBody> {
    from_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal server error",
        None,
    )
}

#[derive(Serialize)]
struct JsonError<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<&'a str>,
}

fn from_error<'a>(
    status: StatusCode,
    error: &'a str,
    data: Option<&'a str>,
) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
    HttpResponseBuilder::new(status).json(JsonError { error, data })
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DbError::ActixBlocking(err) => write!(f, "{}", err),
            DbError::Diesel(err) => write!(f, "{}", err),
            DbError::R2d2(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for DbError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            DbError::ActixBlocking(err) => Some(err),
            DbError::Diesel(err) => Some(err),
            DbError::R2d2(err) => Some(err),
        }
    }
}

impl From<diesel::result::Error> for DbError {
    fn from(err: diesel::result::Error) -> DbError {
        DbError::Diesel(err)
    }
}

impl From<r2d2::Error> for DbError {
    fn from(err: r2d2::Error) -> DbError {
        DbError::R2d2(err)
    }
}

impl From<actix_web::error::BlockingError> for DbError {
    fn from(err: actix_web::error::BlockingError) -> DbError {
        DbError::ActixBlocking(err)
    }
}
