extern crate dotenv;
extern crate futures;
extern crate serde;
extern crate tokio;
extern crate twitch_api;

use futures::future::Future;
use std::env;
use twitch_api::Client;

fn main() {
    dotenv::dotenv().unwrap();
    let twitch_api = Client::new(&env::var("TWITCH_API").unwrap());

    /*
    let users = twitch_api
        .users(vec![], vec!["shroud", "ninja"])
        .and_then(|json| {
            println!("{:?}", json);
            println!("len {}", json.data.len());
            Ok(json)
        })
        .map(|_| ())
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });

    let videos = twitch_api
        .videos(None, Some("37402112"), None)
        .and_then(|json| {
            println!("{:?}", json);
            Ok(json)
        })
        .map(|_| ())
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });
        */
        

    let clip = twitch_api
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .and_then(|json| {
            println!("{:?}", json);
            Ok(json)
        })
        .map(|_| ())
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });

    tokio::run(clip);
}
