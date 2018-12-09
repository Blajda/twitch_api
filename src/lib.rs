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

    pub fn users(&mut self, id: Vec<&str>, login: Vec<&str>) -> Box<Future<Item=(), Error=()> + Send> {
        let mut headers = header::HeaderMap::new();
        let auth_key = &self.client_id;
        let header_value = header::HeaderValue::from_str(&auth_key).unwrap();
        headers.insert("Client-ID", header_value);
        let mut url = String::from("https://") + &String::from(API_DOMAIN) + &String::from("/helix/users");

        if id.len() > 0 || login.len() > 0 {
            url.push_str("?");
        }

        if id.len() > 0 {
            url.push_str("id=");
            for index in 0..id.len() {
                if index != id.len() - 1 {
                    url.push_str(id[index]);
                    url.push(',');
                } else {
                    url.push_str(id[index]);
                }
            }
        }

        if id.len() > 0 && login.len() > 0 {
            url.push_str("&");
        }

        if login.len() > 0 {
            url.push_str("login=");
            for index in 0..login.len() {
                if index != login.len() - 1 {
                    url.push_str(login[index]);
                    url.push(',');
                } else {
                    url.push_str(login[index]);
                }
            }
        }

        let client = Client::builder()
                        .default_headers(headers)
                        .build().unwrap();
        

        let mut response = client
                        .get(&url)
                        .send();

        let f = response
            .map_err(|_| ())
            .and_then(|res| {
                let decoder = res.into_body();
                decoder.collect()
                .map(|chunks| {
                    let mut data: Vec<u8> = Vec::new();
                    for chunk in chunks {
                        for byte in chunk {
                            data.push(byte);
                        }
                    }
                    data
                })
                .map(|data: Vec<u8>| {
                    let s = String::from_utf8_lossy(&data[..]);
                    let j = serde_json::from_str::<serde_json::Value>(&s);
                    println!("{:?}", j);
                    ()
                })
                .map_err(|_| ())
            });

        return Box::new(f);
    }
}
