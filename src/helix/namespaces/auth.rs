use futures::future::Future;
use std::collections::BTreeMap;
use super::super::models::Credentials;
use super::super::Client; 
const ID_DOMAIN: &'static str = "id.twitch.tv";
use super::super::Namespace;

pub struct Auth {}
type AuthNamespace = Namespace<Auth>;

impl AuthNamespace {
    pub fn client_credentials(self, secret: &str) 
        -> ApiRequest<Credentials> {
            use self::client_credentials;
            client_credentials(self.client, secret)
        }
}

impl Client {
    pub fn auth(&self) -> AuthNamespace {
        AuthNamespace::new(self)
    }
}

use super::super::ApiRequest;
use reqwest::Method;

//TODO: Implement scopes
pub fn client_credentials(client: Client, secret: &str)
    -> ApiRequest<Credentials> {

    let url =
        String::from("https://") + 
        ID_DOMAIN + "/oauth2/token";

    let mut params = BTreeMap::new();
    params.insert("client_id".to_owned(), client.id().to_owned());
    params.insert("client_secret".to_owned(), secret.to_owned());
    params.insert("grant_type".to_owned(), "client_credentials".to_owned());
    params.insert("scope".to_owned(), "".to_owned());
    
    ApiRequest::new(url, params, client, Method::POST, None)
}
