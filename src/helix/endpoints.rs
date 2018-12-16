use futures::future::Future;
use reqwest::header;
use reqwest::r#async::{RequestBuilder};
use reqwest::r#async::Client as ReqwestClient;
use std::sync::Arc;

use super::models::{DataContainer, PaginationContainer, User, Video, Clip};
use super::Client; 
const API_DOMAIN: &'static str = "api.twitch.tv";

/* When Client owns a ReqwestClient, any futures spawned do not immediately
 * terminate but 'hang'. When creating a new client for each request this problem
 * does not occur. This would need to be resolved so we can benefit from keep alive
 * connections.
 */

use super::super::GetRequest;
use super::super::GetRequestRef;
use std::marker::PhantomData;

use super::HelixClient;

fn apply_standard_headers(client: &Box<dyn HelixClient>, request: RequestBuilder) 
   -> RequestBuilder 
{
    let client_header = header::HeaderValue::from_str(client.id()).unwrap();

    request.header("Client-ID", client_header)
}

pub fn clip(client: &Box<dyn HelixClient>, id: &str) 
    -> impl Future<Item=DataContainer<Clip>, Error=reqwest::Error>
{
    let url =
        String::from("https://") + 
        API_DOMAIN + "/helix/clips" + "?id=" + id;


    let request = client.client().get(&url);
    let request = apply_standard_headers(client, request);

    request
        .send()
        .map(|mut res| {
            println!("{:?}", res);
            res.json::<DataContainer<Clip>>()
        })
        .and_then(|json| json)
            /*
    GetRequest {
        inner: Arc::new(GetRequestRef {
            url: url.to_owned(),
            returns: PhantomData,
        })
    }
    */
}

impl Client {



/*
    pub fn users(
        &self,
        id: Vec<&str>,
        login: Vec<&str>,
    ) -> impl Future<Item = DataContainer<User>, Error = reqwest::Error> {
        let mut url =
            String::from("https://") + &String::from(API_DOMAIN) + &String::from("/helix/users");

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


        let f = self.create_client()
            .get(&url)
            .send()
            .map(|mut res| res.json::<DataContainer<User>>())
            .and_then(|json| json);

        return f;
    }

    pub fn videos(
        &self,
        video_id:   Option<Vec<&str>>,
        user_id:    Option<&str>,
        game_id:    Option<&str>,
    ) -> impl Future<Item = PaginationContainer<Video>, Error = reqwest::Error> {
        let mut url =
            String::from("https://") + &String::from(API_DOMAIN) + &String::from("/helix/videos");

        url.push_str("?");
        if let Some(user_id) = user_id {
            url.push_str("user_id=");
            url.push_str(user_id);
            url.push('&');
        }

        let f = self.create_client()
            .get(&url)
            .send()
            .map(|mut res| {
                res.json::<PaginationContainer<Video>>()
            })
            .and_then(|json| json);

        return f;
    }
*/

}

