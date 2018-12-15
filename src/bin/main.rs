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
    let client_id = &env::var("TWITCH_API").unwrap();
    let client =  Client::new(client_id);

    let clip = client.helix
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });

    let clip2 = client.kraken
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });

    tokio::run(
        clip.join(clip2)
            .and_then(|(c1, c2)| {
                println!("{:?} {:?}", c1, c2);
                Ok((c1, c2))
            })
            .map(|_| { client.nop(); ()})
            .map_err(|_| ())
    );
}
