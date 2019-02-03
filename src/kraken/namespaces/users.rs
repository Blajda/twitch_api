use std::collections::BTreeMap;
use super::super::models::{User};
use super::super::Client; 
use crate::client::{RatelimitKey, ClientTrait, ApiRequest};
use reqwest::Method;
use super::Namespace;

pub struct Users {}
type UsersNamespace = Namespace<Users>;

impl UsersNamespace {
    pub fn by_id(self, id: &str) -> ApiRequest<User> {
        use self::by_id;
        by_id(self.client, id)
    }
}

impl Client {
    pub fn users(&self) -> UsersNamespace {
        UsersNamespace::new(self)
    }
}

pub fn by_id(client: Client, id: &str) 
    -> ApiRequest<User>
{
    let client = client.inner;
    let url = String::from("https://") + client.domain() + "/kraken/users/" + id;
    let params  = BTreeMap::new();

    ApiRequest::new(url, params, client, Method::GET, Some(RatelimitKey::Default))
}
