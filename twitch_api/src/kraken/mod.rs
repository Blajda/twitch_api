use crate::client::Client as GenericClient;
use crate::client::{Version, ClientConfig};
use crate::client::ClientTrait;

use crate::client::{KrakenScope, Scope};

mod namespaces;
pub mod models;

#[derive(Clone)]
pub struct Client {
    inner: GenericClient
}

impl Client {
    pub fn new(id: &str) -> Client {
        let config = ClientConfig::default();
        Client {
            inner: GenericClient::new(id, config, Version::Kraken)
        }
    }

    pub fn new_with_config(id: &str, config: ClientConfig) -> Client {
        Client {
            inner: GenericClient::new(id, config, Version::Kraken)
        }
    }

    pub fn authenticate(self, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder::new(self, secret)
    }

    pub fn id<'a>(&'a self) -> &'a str { &self.inner.id() }
    pub fn domain<'a>(&'a self) -> &'a str { &self.inner.domain() }
    pub fn auth_domain<'a>(&'a self) -> &'a str { &self.inner.auth_domain() }
    pub fn authenticated(&self) -> bool { self.inner.authenticated() }

    pub fn scopes(&self) -> Vec<KrakenScope> {
        self.inner.scopes().into_iter().filter_map(|item| {
            if let Scope::Kraken(scope) = item { Some(scope) } else { None }
        }).collect()
    }
}

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
