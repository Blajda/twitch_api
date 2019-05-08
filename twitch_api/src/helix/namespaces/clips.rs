use super::*;
use super::models::{DataContainer, Clip};

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;

impl ClipsNamespace {
    pub fn clip<S: ToString>(self, id: &S) -> ApiRequest<DataContainer<Clip>> {
        clip(self.client, id)
    }
}

impl Client {
    pub fn clips(&self) -> ClipsNamespace {
        ClipsNamespace::new(self)
    }
}


pub fn clip<S: ToString>(client: Client, id: &S) 
    -> ApiRequest<DataContainer<Clip>>
{
    let client = client.inner;
    let url =
        String::from("https://") + 
        client.domain() + "/helix/clips" + "?id=" + &id.to_string();

    let params : ParamList = BTreeMap::new();

    ApiRequest::new(url, params, client, Method::GET, Some(RatelimitKey::Default))
}
