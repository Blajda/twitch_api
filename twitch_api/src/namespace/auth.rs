use std::collections::BTreeMap;
use crate::models::Credentials;
use crate::client::Client; 
use crate::client::{ClientTrait, ApiRequest};
use reqwest::Method;
use std::marker::PhantomData;

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
        -> ApiRequest<Credentials> {
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
    -> ApiRequest<Credentials> {
    //TODO: Implement scopes

    let url = client.auth_base_uri().to_owned() + "/oauth2/token";

    let mut params : BTreeMap<&str, &dyn ToString> = BTreeMap::new();
    let client_id = &client.id();
    params.insert("client_id", &client_id);
    params.insert("client_secret", secret);
    params.insert("grant_type", &"client_credentials");
    params.insert("scope", &"");
    
    ApiRequest::new(url, params, client.clone(), Method::POST, None)
}
