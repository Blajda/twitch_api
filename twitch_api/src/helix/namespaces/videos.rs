use super::models::{PaginationContainer, Video};
use super::*;

pub struct Videos {}
type VideosNamespace = Namespace<Videos>;
/*
impl VideosNamespace {
    pub fn by_id<S: ToString>(self, ids: &[S])
        -> IterableApiRequest<PaginationContainer<Video>> {
        by_id(self.client, ids)
    }

    pub fn by_user<S: ToString>(self, user_id: &S)
        -> IterableApiRequest<PaginationContainer<Video>> {
        by_user(self.client, user_id)
    }

    pub fn for_game<S: ToString>(self, game_id: &S)
        -> IterableApiRequest<PaginationContainer<Video>> {
        for_game(self.client, game_id)
    }
}

impl Client {

    pub fn videos(&self) -> VideosNamespace {
        VideosNamespace::new(self)
    }
}

pub fn by_id<S: ToString>(client: Client, ids: &[S])
    -> IterableApiRequest<PaginationContainer<Video>> {
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + &String::from("/helix/videos");

    let mut params: ParamList = BTreeMap::new();
    for id in ids {
        params.insert("id", id);
    }

    IterableApiRequest::new(url, params, client,
                            Method::GET, Some(RatelimitKey::Default))
}

pub fn by_user<S: ToString>(client: Client, user_id: &S)
    -> IterableApiRequest<PaginationContainer<Video>> {
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + &String::from("/helix/videos");

    let mut params: ParamList = BTreeMap::new();
    params.insert("user_id", user_id);

    IterableApiRequest::new(url, params, client,
                            Method::GET, Some(RatelimitKey::Default))
}

pub fn for_game<S: ToString>(client: Client, game_id: &S)
    -> IterableApiRequest<PaginationContainer<Video>> {
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + &String::from("/helix/videos");

    let mut params: ParamList = BTreeMap::new();
    params.insert("game_id", game_id);

    IterableApiRequest::new(url, params, client,
                            Method::GET, Some(RatelimitKey::Default))
}
*/
