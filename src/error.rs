use core::fmt;
use std::error;

#[derive(Debug)]
pub enum Error {
    ActixBlockingError(actix_web::error::BlockingError),
    DieselError(diesel::result::Error),
    R2d2Error(r2d2::Error),
}

impl From<Error> for actix_web::error::Error {
    fn from(err: Error) -> Self {
        use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound};
        use diesel::result::Error::{
            AlreadyInTransaction, DatabaseError, DeserializationError, NotFound, QueryBuilderError,
            SerializationError,
        };
        use Error::*;
        match err {
            ActixBlockingError(actix_error) => ErrorInternalServerError(actix_error),
            R2d2Error(r2d2_error) => ErrorInternalServerError(r2d2_error),
            DieselError(diesel_error) => match diesel_error {
                DatabaseError(_, _) => ErrorInternalServerError(diesel_error),
                NotFound => ErrorNotFound(diesel_error),
                QueryBuilderError(_) => ErrorBadRequest(diesel_error),
                SerializationError(_) | DeserializationError(_) => ErrorBadRequest(diesel_error),
                AlreadyInTransaction => ErrorInternalServerError(diesel_error),
                _ => ErrorInternalServerError(diesel_error),
            },
        }
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
