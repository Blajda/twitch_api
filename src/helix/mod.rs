pub mod models;
pub mod namespaces;

use crate::client::Client as GenericClient;
use crate::client::AuthClientBuilder;
use crate::client::Version;
use crate::client::ClientTrait;

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
        AuthClientBuilder::new(self.inner, secret)
    }
}
