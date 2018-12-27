extern crate serde_json;

use crate::client::PaginationTrait;

impl PaginationTrait for Credentials {
    fn cursor<'a>(&'a self) -> Option<&'a str> { None }
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
    pub scope: Option<Vec<String>>,
    pub token_type: String,
}
