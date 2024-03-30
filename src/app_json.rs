// https://github.com/rambler-digital-solutions/actix-web-validator/blob/master/src/json.rs
use std::{ops::Deref, sync::Arc};

use actix_web::{
    dev::Payload,
    web::{self, JsonBody},
    FromRequest, HttpRequest,
};
use futures::{future::LocalBoxFuture, FutureExt};
use sanitizer::Sanitize;
use serde::de::DeserializeOwned;
use validator::Validate;

use super::error::AppError;

#[derive(Debug)]
pub struct AppJson<T>(pub T);

impl<T> AppJson<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for AppJson<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for AppJson<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

// https://github.com/rambler-digital-solutions/actix-web-validator/blob/master/src/json.rs
impl<T> FromRequest for AppJson<T>
where
    T: DeserializeOwned + Validate + Sanitize + 'static,
{
    type Error = AppError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let mut helper_container: Option<JsonConfig> = None;
        let config = req
            .app_data::<web::Data<JsonConfig>>()
            .map(|d| d.as_ref())
            .unwrap_or_else(|| helper_container.insert(JsonConfig::default()));

        let JsonConfig {
            limit,
            content_type,
            ..
        } = config;

        JsonBody::new(req, payload, content_type.as_deref(), false)
            .limit(*limit)
            .map(|res: Result<T, _>| match res {
                Ok(mut data) => {
                    data.sanitize();
                    data.validate()
                        .map(|_| AppJson(data))
                        .map_err(AppError::from)
                }
                Err(e) => Err(AppError::from(e)),
            })
            .boxed_local()
    }
}

type ErrHandler = Arc<dyn Fn(AppError, &HttpRequest) -> actix_web::Error + Send + Sync>;

#[derive(Clone)]
pub struct JsonConfig {
    limit: usize,
    ehandler: Option<ErrHandler>,
    content_type: Option<Arc<dyn Fn(mime::Mime) -> bool + Send + Sync>>,
}

impl JsonConfig {
    /// Change max size of payload. By default max size is 32Kb
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set custom error handler
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(AppError, &HttpRequest) -> actix_web::Error + Send + Sync + 'static,
    {
        self.ehandler = Some(Arc::new(f));
        self
    }

    /// Set predicate for allowed content types
    pub fn content_type<F>(mut self, predicate: F) -> Self
    where
        F: Fn(mime::Mime) -> bool + Send + Sync + 'static,
    {
        self.content_type = Some(Arc::new(predicate));
        self
    }
}

impl Default for JsonConfig {
    fn default() -> Self {
        JsonConfig {
            limit: 2_097_152,
            ehandler: None,
            content_type: None,
        }
    }
}

