use futures::future::Future;
use super::super::models::{DataContainer, PaginationContainer, User, Video, Clip};
use super::super::Client; 
const API_DOMAIN: &'static str = "api.twitch.tv";
use super::super::Namespace;

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;

impl ClipsNamespace {
    pub fn clip(self, id: &str) -> impl Future<Item=DataContainer<Clip>, Error=reqwest::Error> {
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
    -> impl Future<Item=DataContainer<Clip>, Error=reqwest::Error>
{
    let url =
        String::from("https://") + 
        API_DOMAIN + "/helix/clips" + "?id=" + id;


    let request = client.client().get(&url);
    let request = client.apply_standard_headers(request);

    request
        .send()
        .map(|mut res| {
            println!("{:?}", res);
            res.json::<DataContainer<Clip>>()
        })
        .and_then(|json| json)
}
