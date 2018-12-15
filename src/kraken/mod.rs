use reqwest::header;
use std::sync::Arc;
use reqwest::r#async::RequestBuilder;
use reqwest::r#async::Client as ReqwestClient;
pub use super::types;

mod endpoints;
mod models;


const ACCEPT: &str = "application/vnd.twitchtv.v5+json";
pub const API_DOMAIN: &str = "api.twitch.tv";

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientRef>,
}

struct ClientRef {
    id: String,
    client: ReqwestClient
}

impl Client {
    pub fn new(id: &str) -> Client {
        Client {
            inner: Arc::new(ClientRef {
                id: id.to_owned(),
                client: ReqwestClient::new(),
            })
        }
    }

    pub fn new_with_client(id: &str, client: ReqwestClient) -> Client {
        Client {
            inner: Arc::new(ClientRef {
                id: id.to_owned(),
                client: client,
            })
        }
    }

   fn apply_standard_headers(&self, request: RequestBuilder) 
       -> RequestBuilder 
    {
        let client_header = header::HeaderValue::from_str(&self.inner.id).unwrap();
        let accept_header = header::HeaderValue::from_str(ACCEPT).unwrap();

        request
            .header("Accept", accept_header)
            .header("Client-ID", client_header)
    }
}
