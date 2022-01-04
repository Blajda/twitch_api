extern crate dotenv;
extern crate futures;
extern crate serde;
extern crate tokio;
extern crate twitch_api;
extern crate env_logger;

use futures::future::Future;
use twitch_types::BroadcasterId;
use std::env;
use twitch_api::HelixClient;
use twitch_api::ClientConfig;
use twitch_api::client::RatelimitMap;
use twitch_types::UserId;

use twitch_api::client::PaginationTrait2;


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
    let client_secret = &env::var("TWITCH_SECRET").unwrap();
    let helix_client =  HelixClient::new_with_config(client_id, config)
        .authenticate(client_secret)
        .build()
        .await
        .unwrap();

    let res = helix_client
        .users()
        .users(&[], &["penta"])
        .await
        .unwrap();
    println!("{:?}", res);
    println!("-----------------------------------");

    let res = helix_client
        .clips()
        .by_broadcaster(&BroadcasterId::new("84316241".to_owned()), None)
        .await
        .unwrap();

    println!("{:?}", res);
    println!("-----------------------------------");

    let res = helix_client
        .channels()
        .channel(&BroadcasterId::new("84316241".to_owned()))
        .await
        .unwrap();
    println!("{:?}", res);
    println!("-----------------------------------");

    let mut pages = 0;
    let mut b = helix_client
        .streams()
        .get()
        .with_query("first", "100");
    let mut res = Some(b.build_iterable());
    while let Some(request) = res {
        let page = request
        .await
        .unwrap();
        println!("{:?}", page);
        println!("-----------------------------------");
        pages = pages + 1;
        res = page.next(); 
    }

    //println!("Total pages: {}", pages);
}
