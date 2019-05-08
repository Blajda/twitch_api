use super::*;
use crate::models::Credentials;
use crate::namespace::auth;

pub struct Auth {}
type AuthNamespace = Namespace<Auth>;

impl AuthNamespace {
    pub fn client_credentials(self, secret: &str) 
        -> ApiRequest<Credentials> {
            auth::client_credentials(self.client.inner, &secret)
        }
}

impl Client {
    pub fn auth(&self) -> AuthNamespace {
        AuthNamespace::new(self)
    }
}
