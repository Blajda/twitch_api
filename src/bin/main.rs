extern crate twitch_api;
extern crate tokio;
extern crate dotenv;
extern crate futures;
extern crate serde;

use twitch_api::TwitchApi;
use std::env;
use futures::future::Future;

fn main() {
    dotenv::dotenv().unwrap();
    let mut twitch_api = TwitchApi::new(env::var("TWITCH_API").unwrap());
    let mut users = twitch_api.users(vec![], vec!["shroud"])
        .and_then(|json| {
            println!("{:?}", json);
            Ok(json)
        })
        .map(|_| ())
        .map_err(|_| ());

    tokio::run(users);
}
