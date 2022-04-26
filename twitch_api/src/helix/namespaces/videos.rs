use twitch_types::{GameId, UserId};

use crate::client::{DefaultOpts, RequestBuilder};

use super::models::{ApiError, PaginationContainer, Video};
use super::*;

pub struct Videos {}
type VideosNamespace = Namespace<Videos>;

impl<T, E> RequestBuilder<T, E, Videos> {
    ///Language of the video being queried. Limit: 1. A language value must be
    ///either the ISO 639-1 two-letter code for a supported stream language or
    ///“other”.
    pub fn language<S: Into<String>>(self, lang: S) -> Self {
        self.with_query("language", lang)
    }

    ///Period during which the video was created. Valid values: "all", "day",
    ///"week", "month". Default: "all".
    pub fn period<S: Into<String>>(self, period: S) -> Self {
        self.with_query("period", period)
    }

    ///Sort order of the videos. Valid values: "time", "trending", "views".
    ///Default: "time".
    pub fn sort<S: Into<String>>(self, sort: S) -> Self {
        self.with_query("sort", sort)
    }

    ///Type of video. Valid values: "all", "upload", "archive", "highlight".
    ///Default: "all".
    pub fn r#type<S: Into<String>>(self, t: S) -> Self {
        self.with_query("type", t)
    }
}

impl VideosNamespace {
    pub fn by_id<S: ToString>(self, ids: &[S]) -> RequestBuilder<PaginationContainer<Video>> {
        by_id(self.client, ids)
    }

    pub fn by_user<'a, S: Into<UserId<'a>>>(
        self,
        user_id: S,
    ) -> RequestBuilder<PaginationContainer<Video>, ApiError, Videos> {
        by_user(self.client, user_id)
    }

    pub fn for_game<'a, S: Into<GameId<'a>>>(
        self,
        game_id: S,
    ) -> RequestBuilder<PaginationContainer<Video>, ApiError, Videos> {
        for_game(self.client, game_id)
    }
}

impl Client {
    pub fn videos(&self) -> VideosNamespace {
        VideosNamespace::new(self)
    }
}

pub fn by_id<S: ToString>(
    client: Client,
    ids: &[S],
) -> RequestBuilder<PaginationContainer<Video>, ApiError, DefaultOpts> {
    let url = client.inner.api_base_uri().to_owned() + &String::from("/videos");
    let mut b = RequestBuilder::new(client.inner, url, Method::GET);

    for id in ids {
        b = b.with_query("id", id.to_string());
    }

    b
}

pub fn by_user<'a, Id: Into<UserId<'a>>>(
    client: Client,
    user_id: Id,
) -> RequestBuilder<PaginationContainer<Video>, ApiError, Videos> {
    let url = client.inner.api_base_uri().to_owned() + &String::from("/videos");
    let mut b = RequestBuilder::new(client.inner, url, Method::GET);

    b = b.with_query("user_id", user_id.into());

    b
}

pub fn for_game<'a, Id: Into<GameId<'a>>>(
    client: Client,
    game_id: Id,
) -> RequestBuilder<PaginationContainer<Video>, ApiError, Videos> {
    let url = client.inner.api_base_uri().to_owned() + &String::from("/videos");
    let mut b = RequestBuilder::new(client.inner, url, Method::GET);

    b = b.with_query("game_id", game_id.into());

    b
}
