use reqwest::Error as ReqwestError;
use futures::future::SharedError;
use std::convert::From;

/*TODO: How should condition errors be handled?
 * Ultimately the future must resolve so if the condition
 * errs than all it's waiters must err.
 */
#[derive(Clone, Debug)]
pub struct ConditionError{}

impl From<SharedError<ConditionError>> for ConditionError {
    fn from(other: SharedError<ConditionError>) -> Self {
        ConditionError{}
    }
}

#[derive(Debug)]
enum Kind {
    Reqwest(ReqwestError),
    ClientError(String),
}

#[derive(Debug)]
pub struct Error {
    inner: Kind
}


impl From<reqwest::Error> for Error {

    fn from(err: ReqwestError) -> Error {
        Error {
            inner: Kind::Reqwest(err)
        }
    }
}

impl From<()> for Error {

    fn from(err: ()) -> Error {
        Error {
            inner: Kind::ClientError("Internal error".to_owned())
        }
    }
}

impl From<futures::Canceled> for Error {

    fn from(_err: futures::Canceled) -> Error {
        Error {
            inner: Kind::ClientError("Oneshot channel unexpectedly closed".to_owned())
        }
    }
}

use std::sync::mpsc::SendError;

impl<T> From<SendError<T>> for Error {

    fn from(_err: SendError<T>) -> Error {
        Error {
            inner: Kind::ClientError("Channel unexpectedly closed".to_owned())
        }
    }
}

impl From<ConditionError> for Error {

    fn from(_err: ConditionError) -> Error {
        Error {
            inner: Kind::ClientError("Oneshot channel unexpectedly closed".to_owned())
        }
    }
}
