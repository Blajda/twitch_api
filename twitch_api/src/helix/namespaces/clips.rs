use chrono::{DateTime, Utc};
use twitch_types::{BroadcasterId, GameId, UserId};

use crate::client::{DefaultOpts, RequestBuilder};

use super::models::{Clip, DataContainer};
use super::*;

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
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
        time_range: Option<TimeRange>,
    ) -> RequestBuilder<DataContainer<Clip>, DefaultOpts> {
        by_game(self.client, id, time_range)
    }

    ///Get clips for a broadcaster with an optional time range
    ///
    ///Results are ordered by view count
    ///
    ///<https://dev.twitch.tv/docs/api/reference#get-clips>
    pub fn by_broadcaster<'a, Id: Into<BroadcasterId<'a>>>(
        self,
        id: Id,
        time_range: Option<TimeRange>,
    ) -> RequestBuilder<DataContainer<Clip>, DefaultOpts> {
        by_broadcaster(self.client, id, time_range)
    }

    ///Get a list of clips given by id
    ///
    ///<https://dev.twitch.tv/docs/api/reference#get-clips>
    pub fn by_clips<'a, Id: ToString>(
        self,
        ids: &[Id],
    ) -> RequestBuilder<DataContainer<Clip>, DefaultOpts> {
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
    time_range: Option<TimeRange>,
) -> RequestBuilder<DataContainer<Clip>, DefaultOpts> {
    let url = client.inner.api_base_uri().to_owned() + "/helix/clips";
    let mut b = RequestBuilder::new(client.inner, url, Method::GET);

    if let Some(time) = time_range {
        b = b.with_query("started_at", time.start.to_rfc3339());
        b = b.with_query("ended_at", time.end.to_rfc3339());
    }

    return b;
}

///Get clips for a game with an optional time range

///Results are ordered by view count
///
///<https://dev.twitch.tv/docs/api/reference#get-clips>
pub fn by_game<'a, Id: Into<GameId<'a>>>(
    client: Client,
    id: Id,
    time_range: Option<TimeRange>,
) -> RequestBuilder<DataContainer<Clip>, DefaultOpts> {
    let mut b = init_clips_request_builder(client, time_range);
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
    time_range: Option<TimeRange>,
) -> RequestBuilder<DataContainer<Clip>, DefaultOpts> {
    let mut b = init_clips_request_builder(client, time_range);
    b = b.with_query("broadcaster_id", id.into());
    b
}

///Get a list of clips by their id
///
///<https://dev.twitch.tv/docs/api/reference#get-clips>
pub fn by_clips<'a, Id: ToString>(
    client: Client,
    ids: &[Id],
) -> RequestBuilder<DataContainer<Clip>, DefaultOpts> {
    let mut b = init_clips_request_builder(client, None);
    for id in ids {
        b = b.with_query("id", id.to_string());
    }
    b
}
