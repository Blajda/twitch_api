extern crate reqwest;
extern crate futures;
extern crate serde;

use reqwest::r#async::{Client, Request, Chunk, Response, Decoder};
use reqwest::header;
use futures::future::Future;
use std::iter::Iterator;
use futures::Stream;

const API_DOMAIN: &'static str = "api.twitch.tv";

pub struct TwitchApi {
    client_id: String,
}

impl TwitchApi {
    pub fn new(client_id: String) -> TwitchApi {
        TwitchApi {
            client_id
        }
    }

    pub fn users(&mut self, id: Vec<&str>, login: Vec<&str>) -> Box<Future<Item=serde_json::Value, Error=reqwest::Error> + Send> {
        let mut headers = header::HeaderMap::new();
        let auth_key = &self.client_id;
        let header_value = header::HeaderValue::from_str(&auth_key).unwrap();
        headers.insert("Client-ID", header_value);
        let mut url = String::from("https://") + &String::from(API_DOMAIN) + &String::from("/helix/users");

        if id.len() > 0 || login.len() > 0 {
            url.push_str("?");
        }

        if id.len() > 0 {
            for index in 0..id.len() {
                url.push_str("id=");
                url.push_str(id[index]);
                url.push('&');
            }
        }

        if login.len() > 0 {
            for index in 0..login.len() {
                url.push_str("login=");
                url.push_str(login[index]);
                url.push('&');
            }
        }

        let client = Client::builder()
                        .default_headers(headers)
                        .build().unwrap();
        

        let mut f = client
                        .get(&url)
                        .send()
                        .map(|mut res| {
                            res.json::<serde_json::Value>()
                        })
                        .and_then(|json| {
                            json
                        });

        return Box::new(f);
    }
}
