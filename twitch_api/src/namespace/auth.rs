use crate::client::ClientTrait;
use crate::client::{Client, RequestBuilder};
use crate::helix::models::Credentials;
use hyper::Method;
use std::marker::PhantomData;

pub struct Namespace<T> {
    client: Client,
    _type: PhantomData<T>,
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
    pub fn client_credentials(self, secret: &str) -> RequestBuilder<Credentials> {
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
pub fn client_credentials<S: Into<String>>(
    client: Client,
    secret: S,
) -> RequestBuilder<Credentials> {
    //TODO: Implement scopes

    let url = client.auth_base_uri().to_owned() + "/token";
    let mut b = RequestBuilder::new(client.clone(), url, Method::POST);

    let client_id = client.id();
    b = b
        .with_query("client_id", client_id)
        .with_query("client_secret", secret)
        .with_query("grant_type", "client_credentials")
        .with_query("scope", "");

    b
}
