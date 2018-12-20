use futures::future::Future;
use super::super::models::{DataContainer, PaginationContainer, User, Video, Clip};
use super::super::Client; 
use std::collections::BTreeMap;
const API_DOMAIN: &'static str = "api.twitch.tv";
use super::Namespace;

pub struct Videos {}
type VideosNamespace = Namespace<Videos>;

impl VideosNamespace {
    /*
    pub fn videos(self, video_id) -> impl Future<Item=DataContainer<User>, Error=reqwest::Error> {
        use self::videos;
        users(self.client, id, login)
    }
    */
}

impl Client {

    pub fn videos(&self) -> VideosNamespace {
        VideosNamespace::new(self)
    }
}

/*
pub fn videos(
    client: Client,
    video_id:   Option<Vec<&str>>,
    user_id:    Option<&str>,
    game_id:    Option<&str>,
) -> impl Future<Item = PaginationContainer<Video>, Error = reqwest::Error> {
    let mut url =
        String::from("https://") + &String::from(API_DOMAIN) + &String::from("/helix/videos");

    let mut params  = BTreeMap::new();
    for user in user_id {
        params.insert("user_id", user);
    }

    let request = client.client().get(&url);
    let request = client.apply_standard_headers(request);
    let request = request.query(&params);

    request
        .send()
        .map(|mut res| {
            res.json::<PaginationContainer<Video>>()
        })
        .and_then(|json| json)

}
*/
