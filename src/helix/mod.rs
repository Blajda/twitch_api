pub mod models;
pub mod namespaces;

use crate::client::Client as GenericClient;
use crate::client::Version;
use crate::client::ClientTrait;

/*
#[derive(PartialEq, Hash, Eq, Clone)]
pub enum Scope {
    AnalyticsReadExtensions,
    AnalyticsReadGames,
    BitsRead,
    ClipsEdit,
    UserEdit,
    UserEditBroadcast,
    UserReadBroadcast,
    UserReadEmail,
}
*/
use crate::client::Scope;

#[derive(Clone)]
pub struct Client {
    inner: GenericClient
}

impl Client {
    pub fn new(id: &str) -> Client {
        Client {
            inner: GenericClient::new(id, Version::Helix)
        }
    }

    pub fn authenticate(self, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder::new(self, secret)
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
