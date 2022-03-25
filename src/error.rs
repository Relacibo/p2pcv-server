use core::fmt;
use std::error;

use actix_web::{http::StatusCode, HttpResponse, HttpResponseBuilder};

#[derive(Debug)]
pub enum Error {
    ActixBlockingError(actix_web::error::BlockingError),
    DieselError(diesel::result::Error),
    R2d2Error(r2d2::Error),
}

impl actix_web::ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        use diesel::result::Error::{
            DeserializationError, NotFound, QueryBuilderError, SerializationError,
        };
        use Error::*;
        match self {
            DieselError(diesel_err) => match diesel_err {
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::ActixBlockingError(err) => write!(f, "{}", err),
            Error::DieselError(err) => write!(f, "{}", err),
            Error::R2d2Error(err) => write!(f, "{}", err),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::ActixBlockingError(err) => Some(err),
            Error::DieselError(err) => Some(err),
            Error::R2d2Error(err) => Some(err),
        }
    }
}

impl From<diesel::result::Error> for Error {
    fn from(err: diesel::result::Error) -> Error {
        Error::DieselError(err)
    }
}

impl From<r2d2::Error> for Error {
    fn from(err: r2d2::Error) -> Error {
        Error::R2d2Error(err)
    }
}

impl From<actix_web::error::BlockingError> for Error {
    fn from(err: actix_web::error::BlockingError) -> Error {
        Error::ActixBlockingError(err)
    }
}
