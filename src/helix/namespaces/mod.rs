use std::marker::PhantomData;
use super::Client;

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
