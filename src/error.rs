use reqwest::Error as ReqwestError;
use std::convert::From;

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
