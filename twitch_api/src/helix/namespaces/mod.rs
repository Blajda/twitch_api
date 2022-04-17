use std::marker::PhantomData;

pub use super::models;
pub use super::Client;
pub use crate::client::{ApiRequest, ClientTrait, IterableApiRequest, ParamList, RatelimitKey};
pub use hyper::Method;
pub use std::collections::BTreeMap;

pub mod auth;
pub mod channels;
pub mod clips;
pub mod streams;
pub mod users;
pub mod videos;

pub struct Namespace<T> {
    client: Client,
    _type: PhantomData<T>,
}

impl<T> Namespace<T> {
    pub fn new(client: &Client) -> Self {
        Namespace {
            client: client.clone(),
            _type: PhantomData,
        }
    }
}
