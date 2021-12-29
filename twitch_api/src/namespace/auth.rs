use std::collections::BTreeMap;
use crate::client::{Client, RequestBuilder}; 
use crate::client::{ClientTrait, ApiRequest};
use crate::helix::models::Credentials;
use std::marker::PhantomData;
use hyper::Method;

pub struct Namespace<T> {
    client: Client,
    _type: PhantomData<T>
}

impl<T> Namespace<T> {
    pub fn new(client: &Client) -> Self {
        Namespace {
            client: client.clone(),
            _type: PhantomData,
        }
    }
}

pub struct Auth {}
type AuthNamespace = Namespace<Auth>;

impl AuthNamespace {
    pub fn client_credentials(self, secret: &str) 
        -> RequestBuilder<Credentials> {
            client_credentials(self.client, &secret.to_owned())
        }
}

impl Client {
    pub fn auth(&self) -> AuthNamespace {
        AuthNamespace::new(self)
    }
}

/**
 * https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-client-credentials-flow
*/
pub fn client_credentials<S: ToString>(client: Client, secret: &S)
    -> RequestBuilder<Credentials> {
    //TODO: Implement scopes

    let url = client.auth_base_uri().to_owned() + "/oauth2/token";
    let mut b = RequestBuilder::new(client.clone(), url, Method::POST);

    let client_id = client.id();
    b.with_query("client_id", client_id);
    b.with_query("client_secret", secret);
    b.with_query("grant_type", &"client_credentials");
    b.with_query("scope", &"");
    
    return b;
}
