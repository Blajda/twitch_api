#![feature(option_replace)]
extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate chrono;
#[macro_use] extern crate serde_derive;

pub mod helix;
pub mod kraken;
pub mod types;

pub use self::helix::Client as HelixClient;
pub use self::kraken::Client as KrakenClient;

use reqwest::r#async::Client as ReqwestClient;
use reqwest::header::HeaderMap;
use std::marker::PhantomData;
use std::sync::Arc;
use std::collections::BTreeMap;

pub struct Client {
    pub helix: HelixClient,
    pub kraken: KrakenClient,
}

impl Client {
    pub fn new(client_id: &str) -> Client {
        let client = ReqwestClient::new();
        Client {
            helix:  HelixClient::new_with_client(client_id, client.clone()),
            kraken: KrakenClient::new_with_client(client_id, client.clone()),
        }
    }
}

trait Request<T> {
    fn url(&self) -> String;
    fn headers(&self) -> &HeaderMap;
    fn query(&self) -> &BTreeMap<String, String>;
    fn returns(&self) -> T;
}

pub struct GetRequest<T> {
    inner: Arc<GetRequestRef<T>>,
}

struct GetRequestRef<T> {
    url: String,
//    headers: HeaderMap,
//    query: BTreeMap<String, String>,
    returns: PhantomData<T>, 
}
