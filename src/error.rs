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

impl From<futures::Canceled> for Error {

    fn from(_err: futures::Canceled) -> Error {
        Error {
            inner: Kind::ClientError("Oneshot channel unexpectedly closed".to_owned())
        }
    }
}
