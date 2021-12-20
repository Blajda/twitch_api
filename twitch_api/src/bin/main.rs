extern crate dotenv;
extern crate futures;
extern crate serde;
extern crate tokio;
extern crate twitch_api;
extern crate env_logger;

use futures::future::Future;
use std::env;
use twitch_api::HelixClient;
use twitch_api::ClientConfig;
use twitch_api::client::RatelimitMap;
use twitch_types::UserId;


#[tokio::main]
async fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let config = ClientConfig {
        max_retrys: 0,
        ratelimits: RatelimitMap::default(),
        ..ClientConfig::default()
    };

    let client_id = &env::var("TWITCH_API").unwrap();
    let helix_client =  HelixClient::new_with_config(client_id, config);

    let res = helix_client.users().users(&[], &["freakey"]).await.unwrap();
    println!("{:?}", res);
}
