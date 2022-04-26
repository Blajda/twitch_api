use crate::client::Client as GenericClient;
use crate::client::ClientConfig;
use crate::client::ClientTrait;

use crate::client::HelixScope;

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
            inner: GenericClient::new(id, config),
        }
    }

    pub fn new_with_config<S: Into<String>>(id: S, config: ClientConfig) -> Client {
        Client {
            inner: GenericClient::new(id, config),
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

    pub fn scopes(&self) -> &[HelixScope] {
        self.inner.scopes()
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
            inner: self.inner.scope(scope),
        }
    }

    pub fn scopes(self, scopes: Vec<HelixScope>) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self.inner.scopes(scopes.into_iter().map(|e| e).collect()),
        }
    }

    pub fn token(self, token: &str) -> AuthClientBuilder {
        AuthClientBuilder {
            inner: self.inner.token(token),
        }
    }
}
