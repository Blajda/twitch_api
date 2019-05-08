use std::marker::PhantomData;

pub use super::Client;
pub use crate::client::{RatelimitKey, ClientTrait, ApiRequest, IterableApiRequest, ParamList};
pub use std::collections::BTreeMap;
pub use reqwest::Method;
pub use super::models;

pub mod clips;
pub mod users;
pub mod videos;
pub mod auth;

pub struct Namespace<T> {
    client: Client,
    _type: PhantomData<T>
}

impl<T> Namespace<T> {
    pub fn new(client: &Client) -> Self {
        Namespace {
            client: client.clone(),
            _type: PhantomData,
        }
    }
}
