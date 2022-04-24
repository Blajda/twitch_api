use crate::client::RequestBuilder;

use super::models::{PaginationContainer, Stream};
use super::*;

pub struct StreamMarker {}
type StreamNamespace = Namespace<StreamMarker>;

impl Client {
    pub fn streams(&self) -> StreamNamespace {
        StreamNamespace::new(self)
    }
}

impl StreamNamespace {
    pub fn get(self) -> RequestBuilder<PaginationContainer<Stream>> {
        streams(self.client)
    }
}

pub fn streams(client: Client) -> RequestBuilder<PaginationContainer<Stream>> {
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + "/streams";
    let b = RequestBuilder::new(client, url, Method::GET);

    return b;
}
