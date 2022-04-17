extern crate serde_json;

use crate::client::ForwardPagination;
use crate::client::Scope;

impl ForwardPagination for Credentials {
    fn cursor<'a>(&'a self) -> Option<&'a str> {
        None
    }
}

impl ForwardPagination for Message {
    fn cursor<'a>(&'a self) -> Option<&'a str> {
        None
    }
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
    pub scope: Option<Vec<Scope>>,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub error: Option<String>,
    pub message: String,
    pub status: u32,
}
