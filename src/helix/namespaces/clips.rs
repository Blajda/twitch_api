use futures::future::Future;
use std::collections::BTreeMap;
use super::super::models::{DataContainer, PaginationContainer, User, Video, Clip};
use super::super::Client; 
const API_DOMAIN: &'static str = "api.twitch.tv";
use super::super::Namespace;

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

use super::super::ApiRequest;


pub fn clip(client: Client, id: &str) 
    -> ApiRequest<DataContainer<Clip>>
{
    let url =
        String::from("https://") + 
        API_DOMAIN + "/helix/clips" + "?id=" + id;

    let params  = BTreeMap::new();

    ApiRequest::new( url, params, client)
}
