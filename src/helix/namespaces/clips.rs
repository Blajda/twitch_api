use super::*;
use super::models::{DataContainer, Clip};
use crate::types::ClipId;

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;

impl ClipsNamespace {
    pub fn clip(self, id: &ClipId) -> ApiRequest<DataContainer<Clip>> {
        use self::clip;
        clip(self.client, id)
    }
}

impl Client {
    pub fn clips(&self) -> ClipsNamespace {
        ClipsNamespace::new(self)
    }
}


pub fn clip(client: Client, id: &ClipId) 
    -> ApiRequest<DataContainer<Clip>>
{
    let client = client.inner;
    let url =
        String::from("https://") + 
        client.domain() + "/helix/clips" + "?id=" + id.as_ref();

    let params  = BTreeMap::new();

    ApiRequest::new(url, params, client, Method::GET, Some(RatelimitKey::Default))
}
