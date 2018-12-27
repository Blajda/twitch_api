use std::collections::BTreeMap;
use super::super::models::{Clip};
use super::super::Client; 
use crate::client::{RatelimitKey, ClientTrait, ApiRequest};
use reqwest::Method;
use super::Namespace;

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;

impl ClipsNamespace {
    pub fn clip(self, id: &str) -> ApiRequest<Clip> {
        use self::clip;
        clip(self.client, id)
    }
}

impl Client {

    pub fn clips(&self) -> ClipsNamespace {
        ClipsNamespace::new(self)
    }
}

pub fn clip(client: Client, id: &str) 
    -> ApiRequest<Clip>
{
    let client = client.inner;
    let url = String::from("https://") + client.domain() + "/kraken/clips/" + id;
    let params  = BTreeMap::new();

    ApiRequest::new(url, params, client, Method::GET, Some(RatelimitKey::Default))
}
