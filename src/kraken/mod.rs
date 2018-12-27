use crate::client::Client as GenericClient;
use crate::client::Version;
pub use super::types;

mod namespaces;
pub mod models;

#[derive(Clone)]
pub struct Client {
    inner: GenericClient
}

impl Client {
    pub fn new(id: &str) -> Client {
        Client {
            inner: GenericClient::new(id, Version::Kraken)
        }
    }
}
