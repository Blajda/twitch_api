use crate::client::RequestBuilder;

use super::models::{ApiError, PaginationContainer, Stream};
use super::*;

pub struct Streams {}
type StreamNamespace = Namespace<Streams>;
type StreamBuilder = RequestBuilder<PaginationContainer<Stream>, ApiError, Streams>;

impl Client {
    pub fn streams(&self) -> StreamNamespace {
        StreamNamespace::new(self)
    }
}

impl<T, E> RequestBuilder<T, E, Streams> {
    /// Maximum number of objects to return. Maximum: 100. Default: 20.
    pub fn first(self, first: u32) -> Self {
        self.with_query("first", first.to_string())
    }
}

impl StreamNamespace {
    pub fn get(self) -> StreamBuilder {
        streams(self.client)
    }
}

pub fn streams(client: Client) -> StreamBuilder {
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + "/streams";
    let b = RequestBuilder::new(client, url, Method::GET);

    return b;
}
