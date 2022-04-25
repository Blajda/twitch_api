use crate::client::Client as GenericClient;
use crate::client::ClientTrait;
use crate::client::{ClientConfig, Version};

use crate::client::{HelixScope, Scope};

pub mod limiter;
pub mod models;
pub mod namespaces;

#[derive(Clone, Debug)]
pub struct Client {
    inner: GenericClient,
}

impl Client {
    pub fn new(id: &str) -> Client {
        let config = ClientConfig::default();
        Client {
            inner: GenericClient::new(id, config, Version::Helix),
        }
    }

    pub fn new_with_config<S: Into<String>>(id: S, config: ClientConfig) -> Client {
        Client {
            inner: GenericClient::new(id, config, Version::Helix),
        }
    }

    pub fn authenticate<S: Into<String>>(self, secret: S) -> AuthClientBuilder {
        AuthClientBuilder::new(self, secret)
    }

    pub fn id<'a>(&'a self) -> &'a str {
        &self.inner.id()
    }
    pub fn domain<'a>(&'a self) -> &'a str {
        &self.inner.api_base_uri()
    }
    pub fn auth_domain<'a>(&'a self) -> &'a str {
        &self.inner.auth_base_uri()
    }
    pub fn authenticated(&self) -> bool {
        self.inner.authenticated()
    }

    pub fn scopes(&self) -> Vec<HelixScope> {
        self.inner
            .scopes()
            .into_iter()
            .filter_map(|item| {
                if let Scope::Helix(scope) = item {
                    Some(scope)
                } else {
                    None
                }
            })
            .collect()
    }
}

use crate::client::AuthClientBuilder as GenericAuthClientBuilder;
use crate::error::Error;

pub struct AuthClientBuilder {
    inner: GenericAuthClientBuilder,
}

impl AuthClientBuilder {
    pub fn new<S: Into<String>>(client: Client, secret: S) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: GenericAuthClientBuilder::new(client.inner, secret),
        }
    }

    pub async fn build(self) -> Result<Client, Error> {
        let client = self.inner.build().await?;
        Ok(Client { inner: client })
    }

    pub fn scope(self, scope: HelixScope) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self.inner.scope(Scope::Helix(scope)),
        }
    }

    pub fn scopes(self, scopes: Vec<HelixScope>) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self
                .inner
                .scopes(scopes.into_iter().map(|e| Scope::Helix(e)).collect()),
        }
    }

    pub fn token(self, token: &str) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self.inner.token(token),
        }
    }
}
