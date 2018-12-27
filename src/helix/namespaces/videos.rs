use super::*;
use super::models::{PaginationContainer, Video};
use crate::types::{UserId, GameId, VideoId};


pub struct Videos {}
type VideosNamespace = Namespace<Videos>;

impl VideosNamespace {
    pub fn by_id(self, ids: Vec<&VideoId>)
        -> IterableApiRequest<PaginationContainer<Video>> {
        use self::by_id;
        by_id(self.client, ids)
    }

    pub fn by_user(self, user_id: &UserId)
        -> IterableApiRequest<PaginationContainer<Video>> {
        use self::by_user;
        by_user(self.client, user_id)
    }

    pub fn for_game(self, game_id: &GameId) 
        -> IterableApiRequest<PaginationContainer<Video>> {
        use self::for_game;
        for_game(self.client, game_id)
    }
}

impl Client {

    pub fn videos(&self) -> VideosNamespace {
        VideosNamespace::new(self)
    }
}

pub fn by_id(client: Client, ids: Vec<&VideoId>) 
    -> IterableApiRequest<PaginationContainer<Video>> {
    let client = client.inner;
    let url =
        String::from("https://") + client.domain() + &String::from("/helix/videos");

    let mut params  = BTreeMap::new();
    for id in ids {
        params.insert("id", id.as_ref());
    }

    IterableApiRequest::new(url, params, client,
                            Method::GET, Some(RatelimitKey::Default))
}

pub fn by_user(client: Client, user_id: &UserId) 
    -> IterableApiRequest<PaginationContainer<Video>> {
    let client = client.inner;
    let url =
        String::from("https://") + client.domain() + &String::from("/helix/videos");

    let mut params  = BTreeMap::new();
    params.insert("user_id", user_id.as_ref());

    IterableApiRequest::new(url, params, client,
                            Method::GET, Some(RatelimitKey::Default))
}

pub fn for_game(client: Client, game_id: &GameId) 
    -> IterableApiRequest<PaginationContainer<Video>> {
    let client = client.inner;
    let url =
        String::from("https://") + client.domain() + &String::from("/helix/videos");

    let mut params  = BTreeMap::new();
    params.insert("game_id", game_id.as_ref());

    IterableApiRequest::new(url, params, client,
                            Method::GET, Some(RatelimitKey::Default))
}
