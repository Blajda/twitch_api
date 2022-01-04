use twitch_types::BroadcasterId;

use crate::client::RequestBuilder;

use super::*;
use super::models::{DataContainer, Channel};

pub struct Channels {}
type ChannelNamespace = Namespace<Channels>;

impl ChannelNamespace {
    pub fn channel(self, id: &BroadcasterId) -> RequestBuilder<DataContainer<Channel>> {
        channels(self.client, id)
    }
}

impl Client {
    pub fn channels(&self) -> ChannelNamespace {
        ChannelNamespace::new(self)
    }
}


pub fn channels(client: Client, id: &BroadcasterId) 
    -> RequestBuilder<DataContainer<Channel>>
{
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + "/helix/channels";
    let mut b = RequestBuilder::new(client, url, Method::GET);
    b = b.with_query("broadcaster_id", id);

    return b;
}