use std::collections::BTreeMap;
use super::super::models::Credentials;
use super::super::Client; 
const ID_DOMAIN: &'static str = "id.twitch.tv";
use super::Namespace;
use super::super::ClientTrait;

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
    params.insert("client_id", client.id());
    params.insert("client_secret", secret);
    params.insert("grant_type", "client_credentials");
    params.insert("scope", "");
    
    ApiRequest::new(url, params, client.clone(), Method::POST, None)
}
