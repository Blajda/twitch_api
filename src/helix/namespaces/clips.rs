use std::collections::BTreeMap;
use super::super::models::{DataContainer, Clip};
use super::super::Client; 
use super::Namespace;

use crate::client::{RatelimitKey, ClientTrait, ApiRequest};

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;

impl ClipsNamespace {
    pub fn clip(self, id: &str) -> ApiRequest<DataContainer<Clip>> {
        use self::clip;
        clip(self.client, id)
    }
}

impl Client {

    pub fn clips(&self) -> ClipsNamespace {
        ClipsNamespace::new(self)
    }
}

use reqwest::Method;

pub fn clip(client: Client, id: &str) 
    -> ApiRequest<DataContainer<Clip>>
{
    let client = client.inner;
    let url =
        String::from("https://") + 
        client.domain() + "/helix/clips" + "?id=" + id;

    let params  = BTreeMap::new();

    ApiRequest::new(url, params, client, Method::GET, Some(RatelimitKey::Default))
}
