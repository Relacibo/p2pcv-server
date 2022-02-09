use core::fmt;
use std::error;

#[derive(Debug)]
pub enum Error {
    ActixBlockingError(actix_web::error::BlockingError),
    DieselError(diesel::result::Error),
    R2d2Error(r2d2::Error),
}

impl actix_web::ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        use actix_web::HttpResponse;
        use diesel::result::Error::{
            AlreadyInTransaction, DatabaseError, DeserializationError, NotFound, QueryBuilderError,
            SerializationError,
        };
        use Error::*;
        let mut ret = match self {
            ActixBlockingError(_) => HttpResponse::InternalServerError(),
            R2d2Error(_) => HttpResponse::InternalServerError(),
            DieselError(diesel_err) => match diesel_err {
                DatabaseError(_, _) => HttpResponse::InternalServerError(),
                NotFound => HttpResponse::NotFound(),
                QueryBuilderError(_) => HttpResponse::BadRequest(),
                SerializationError(_) | DeserializationError(_) => HttpResponse::BadRequest(),
                AlreadyInTransaction => HttpResponse::InternalServerError(),
                _ => HttpResponse::InternalServerError(),
            },
        };
        // https://github.com/actix/actix-web/issues/1163
        ret.json(json!({"error": self.to_string()}))
    }
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
