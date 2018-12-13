use reqwest::header;
use reqwest::r#async::{Chunk, Decoder, Request, Response};
use reqwest::r#async::Client as ReqwestClient;

mod endpoints;
mod models;

const ACCEPT: &str = "application/vnd.twitchtv.v5+json";
pub const API_DOMAIN: &str = "api.twitch.tv";

pub struct Client {
    id: String,
}

impl Client {
    pub fn new(id: &str) -> Client {
        Client {
            id: id.to_owned(),
        }
    }

   fn create_reqwest_client(&self) -> ReqwestClient {
        let mut headers = header::HeaderMap::new();
        let auth_key = &self.id;
        let client_header = header::HeaderValue::from_str(auth_key).unwrap();
        let accept_header = header::HeaderValue::from_str(ACCEPT).unwrap();
        headers.insert("Client-ID", client_header);
        headers.insert("Accept", accept_header);

        let client = ReqwestClient::builder().default_headers(headers).build().unwrap();
        client
    }
}
