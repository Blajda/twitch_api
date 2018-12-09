extern crate twitch_api;
extern crate tokio;
extern crate dotenv;

use twitch_api::TwitchApi;
use std::env;

fn main() {
    dotenv::dotenv().unwrap();
    let mut twitch_api = TwitchApi::new(env::var("TWITCH_API").unwrap());
    let users = twitch_api.users(vec![], vec!["shroud"]);

    tokio::run(users);
}
