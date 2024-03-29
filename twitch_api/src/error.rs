use crate::helix::models::ApiError;
use crate::models::Message;
use hyper::Error as HyperError;
use serde_json::Error as JsonError;
use std::convert::From;
use std::error::Error as StdError;
use std::fmt::Display;
use tokio::time::error::Elapsed;

#[derive(Debug)]
pub(crate) enum Kind {
    Hyper(HyperError),
    Io(std::io::Error),
    Json(JsonError),
    AuthError(Option<Message>),
    RatelimitError(Option<Message>),
    RatelimitCostError(String),
    GeneralApiError(ApiError),
    Timeout(Elapsed),
}

#[derive(Debug)]
pub struct Error {
    pub(crate) inner: Kind,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_auth_error() {
            write!(
                f,
                "Not authenticated to Twitch.\n Check credentials and try again"
            )?;
        } else if self.is_ratelimit_error() {
            write!(f, "Twitch ratelimit hit. Try your request again")?;
        } else {
            write!(f, "Unable to perform Twitch API request")?;
        }

        Ok(())
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match &self.inner {
            Kind::Hyper(e) => e.source(),
            Kind::Io(e) => e.source(),
            Kind::Json(e) => e.source(),
            Kind::AuthError(_) => None,
            Kind::RatelimitError(_) => None,
            Kind::RatelimitCostError(_) => None,
            Kind::GeneralApiError(_) => None,
            Kind::Timeout(e) => e.source(),
        }
    }
}

impl Error {
    pub fn auth_error(message: Option<Message>) -> Error {
        Error {
            inner: Kind::AuthError(message),
        }
    }

    pub fn ratelimit_error(message: Option<Message>) -> Error {
        Error {
            inner: Kind::RatelimitError(message),
        }
    }

    pub fn is_auth_error(&self) -> bool {
        match &self.inner {
            Kind::AuthError(_) => true,
            _ => false,
        }
    }

    pub fn is_ratelimit_error(&self) -> bool {
        match &self.inner {
            Kind::RatelimitError(_) => true,
            _ => false,
        }
    }

    pub fn get_api_error(&self) -> Option<&ApiError> {
        match &self.inner {
            Kind::GeneralApiError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Error {
            inner: Kind::Hyper(err),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error {
            inner: Kind::Io(err),
        }
    }
}

impl From<JsonError> for Error {
    fn from(err: JsonError) -> Error {
        Error {
            inner: Kind::Json(err),
        }
    }
}

impl From<Elapsed> for Error {
    fn from(elapse: Elapsed) -> Error {
        return Error {
            inner: Kind::Timeout((elapse)),
        };
    }
}

impl From<ApiError> for Error {
    fn from(err: ApiError) -> Error {
        if err.status == 400 {
            return Error {
                inner: Kind::AuthError(Some(err.into())),
            };
        } else if err.status == 429 {
            return Error {
                inner: Kind::RatelimitError(Some(err.into())),
            };
        }

        Error {
            inner: Kind::GeneralApiError(err),
        }
    }
}
