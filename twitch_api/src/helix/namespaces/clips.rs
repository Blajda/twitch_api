use super::models::{ApiError, Clip, DataContainer};
use super::*;
use crate::client::{DefaultOpts, RequestBuilder};
use twitch_types::{BroadcasterId, GameId, UserId};

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;

impl<T> RequestBuilder<T, Clips> {
    ///Ending date/time for returned clips, in RFC3339 format. (Note that the
    ///seconds value is ignored.) If this is specified, started_at also must be
    ///specified; otherwise, the time period is ignored.
    pub fn ended_at<S: Into<String>>(self, end: S) -> Self {
        self.with_query("ended_at", end)
    }

    ///Starting date/time for returned clips, in RFC3339 format. (The seconds
    ///value is ignored.) If this is specified, ended_at also should be
    ///specified; otherwise, the ended_at date/time will be 1 week after the
    ///started_at value.
    pub fn started_at<S: Into<String>>(self, start: S) -> Self {
        self.with_query("started_at", start)
    }
}

impl ClipsNamespace {
    ///Get clips for a game with an optional time range
    ///
    ///Results are ordered by view count
    ///
    ///<https://dev.twitch.tv/docs/api/reference#get-clips>
    pub fn by_game<'a, Id: Into<GameId<'a>>>(
        self,
        id: Id,
    ) -> RequestBuilder<DataContainer<Clip>, ApiError, Clips> {
        by_game(self.client, id)
    }

    ///Get clips for a broadcaster with an optional time range
    ///
    ///Results are ordered by view count
    ///
    ///<https://dev.twitch.tv/docs/api/reference#get-clips>
    pub fn by_broadcaster<'a, Id: Into<BroadcasterId<'a>>>(
        self,
        id: Id,
    ) -> RequestBuilder<DataContainer<Clip>, ApiError, Clips> {
        by_broadcaster(self.client, id)
    }

    ///Get a list of clips given by id
    ///
    ///<https://dev.twitch.tv/docs/api/reference#get-clips>
    pub fn by_clips<'a, Id: ToString>(
        self,
        ids: &[Id],
    ) -> RequestBuilder<DataContainer<Clip>, ApiError, DefaultOpts> {
        by_clips(self.client, ids)
    }
}

impl Client {
    ///Twitch's Clip Resource
    ///
    ///<https://dev.twitch.tv/docs/api/reference#create-clip>
    pub fn clips(&self) -> ClipsNamespace {
        ClipsNamespace::new(self)
    }
}

fn init_clips_request_builder(
    client: Client,
) -> RequestBuilder<DataContainer<Clip>, ApiError, Clips> {
    let url = client.inner.api_base_uri().to_string() + "clips";
    let b = RequestBuilder::new(client.inner, url, Method::GET);

    return b;
}

///Get clips for a game with an optional time range
///
///Results are ordered by view count
///
///<https://dev.twitch.tv/docs/api/reference#get-clips>
pub fn by_game<'a, Id: Into<GameId<'a>>>(
    client: Client,
    id: Id,
) -> RequestBuilder<DataContainer<Clip>, ApiError, Clips> {
    let mut b = init_clips_request_builder(client);
    b = b.with_query("game_id", id.into());
    b
}

///Get clips for a broadcaster with an optional time range
///
///Results are ordered by view count
///
///<https://dev.twitch.tv/docs/api/reference#get-clips>
pub fn by_broadcaster<'a, Id: Into<UserId<'a>>>(
    client: Client,
    id: Id,
) -> RequestBuilder<DataContainer<Clip>, ApiError, Clips> {
    let mut b = init_clips_request_builder(client);
    b = b.with_query("broadcaster_id", id.into());
    b
}

///Get a list of clips by their id
///
///<https://dev.twitch.tv/docs/api/reference#get-clips>
pub fn by_clips<'a, Id: ToString>(
    client: Client,
    ids: &[Id],
) -> RequestBuilder<DataContainer<Clip>, ApiError, DefaultOpts> {
    let url = client.inner.api_base_uri().to_string() + "/clips";
    let mut b = RequestBuilder::new(client.inner, url, Method::GET);
    for id in ids {
        b = b.with_query("id", id.to_string());
    }
    b
}
