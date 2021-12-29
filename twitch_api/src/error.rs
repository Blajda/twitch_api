use hyper::Error as HyperError;
use std::convert::From;
use std::sync::Arc;
use serde_json::Error as JsonError;
use crate::models::Message;

#[derive(Clone, Debug)]
pub struct ConditionError {
    inner:  Arc<Error>,
}

impl From<Error> for ConditionError {
    fn from(other: Error) -> Self {
        ConditionError{ inner: Arc::new(other) }
    }
}

#[derive(Debug)]
enum Kind {
    Hyper(HyperError),
    ConditionError(ConditionError),
    Io(std::io::Error),
    Json(JsonError),
    AuthError(Option<Message>),
    RatelimitError(Option<Message>),
}

#[derive(Debug)]
pub struct Error {
    inner: Kind
}

impl Error {
    pub fn auth_error(message: Option<Message>) -> Error {
        Error { inner: Kind::AuthError(message) }
    }

    pub fn ratelimit_error(message: Option<Message>) -> Error {
        Error { inner: Kind::RatelimitError(message) }
    }

    pub fn is_auth_error(&self) -> bool {
        match &self.inner {
            Kind::AuthError(_) => true,
            Kind::ConditionError(condition) => condition.inner.is_auth_error(),
            _ => false,
        }
    }

    pub fn is_ratelimit_error(&self) -> bool {
        match &self.inner {
            Kind::RatelimitError(_) => true,
            Kind::ConditionError(condition) => condition.inner.is_ratelimit_error(),
            _ => false,
        }
    }
}


impl From<HyperError> for Error {

    fn from(err: HyperError) -> Error {
        Error {
            inner: Kind::Hyper(err)
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error { inner: Kind::Io(err) }
    }
}

impl From<JsonError> for Error {

    fn from(err: JsonError) -> Error {
        Error { inner: Kind::Json(err) }
    }
}

impl From<ConditionError> for Error {

    fn from(err: ConditionError) -> Error {
        Error { inner: Kind::ConditionError(err) }
    }
}
