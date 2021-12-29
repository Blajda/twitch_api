use twitch_types::{ClipId, BroadcasterId, GameId};

use crate::client::RequestBuilder;

use super::*;
use super::models::{DataContainer, Clip};

pub struct Clips {}
type ClipsNamespace = Namespace<Clips>;
pub struct TimeRange {}

pub enum ClipRequest<'a> {
    ByClip(&'a [&'a ClipId]),
    ByBroadcaster(&'a BroadcasterId),
    ByGame(&'a GameId),
}

impl<'a> From<&'a BroadcasterId> for ClipRequest<'a> {
    fn from(id: &'a BroadcasterId) -> Self {
        return ClipRequest::ByBroadcaster(id);
    }
}

impl<'a> From<&'a GameId> for ClipRequest<'a> {
    fn from(id: &'a GameId) -> Self {
        return ClipRequest::ByGame(id);
    }
}

impl ClipsNamespace {
    pub fn clip(self, id: ClipRequest, time_range: Option<TimeRange>) 
    -> RequestBuilder<DataContainer<Clip>> {
        clip(self.client, id, time_range)
    }

    pub fn by_game(self, id: &GameId, time_range: Option<TimeRange>) 
    -> RequestBuilder<DataContainer<Clip>> {
        by_game(self.client, id, time_range)
    }

    pub fn by_broadcaster(self, id: &BroadcasterId, time_range: Option<TimeRange>) 
    -> RequestBuilder<DataContainer<Clip>> {
        by_broadcaster(self.client, id, time_range)
    }

    pub fn by_clips(self, ids: &[&ClipId]) 
    -> RequestBuilder<DataContainer<Clip>> {
        by_clips(self.client, ids)
    }
}

impl Client {
    pub fn clips(&self) -> ClipsNamespace {
        ClipsNamespace::new(self)
    }
}

pub fn clip(client: Client, id: ClipRequest, time_range: Option<TimeRange>) 
    -> RequestBuilder<DataContainer<Clip>>
{
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + "/helix/clips";
    let mut b = RequestBuilder::new(client, url, Method::GET);

    match id {
        ClipRequest::ByClip(ids) => {
            for id in ids {
                todo!("implement me")
                //params.insert("id", id);
            }
        },
        ClipRequest::ByBroadcaster(id) => {
            b.with_query("broadcaster_id", id);
        },
        ClipRequest::ByGame(id) => {
            b.with_query("game_id", id);
        }
    } 
    return b;
}

pub fn by_game(client: Client, id: &GameId, time_range: Option<TimeRange>) 
-> RequestBuilder<DataContainer<Clip>>
{
    clip(client, id.into(), time_range)
}

pub fn by_broadcaster(client: Client, id: &BroadcasterId, time_range: Option<TimeRange>) 
-> RequestBuilder<DataContainer<Clip>>
{
    clip(client, id.into(), time_range)
}

pub fn by_clips(client: Client, ids: &[&ClipId]) 
-> RequestBuilder<DataContainer<Clip>>
{
    clip(client, ClipRequest::ByClip(ids.into()), None)
}
