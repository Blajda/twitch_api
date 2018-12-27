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

    pub fn authenticate(self, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder::new(self, secret)
    }
}

use crate::client::Scope;
use crate::client::AuthClientBuilder as GenericAuthClientBuilder;

pub struct AuthClientBuilder {
    inner: GenericAuthClientBuilder,
}

impl AuthClientBuilder {
    pub fn new(client: Client, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: GenericAuthClientBuilder::new(client.inner, secret), 
        }
    }

    pub fn build(self) -> Client {
        let client = self.inner.build();
        Client {
            inner: client
        }
    }

    pub fn scope(self, scope: Scope) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self.inner.scope(scope)
        }
    }

    pub fn scopes(self, scopes: Vec<Scope>) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self.inner.scopes(scopes)
        }
    }

    pub fn token(self, token: &str) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self.inner.token(token)
        }
    }
}
