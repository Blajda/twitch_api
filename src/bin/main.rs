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


    let clip = client.helix.clips()
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

    /* Prevents tokio from **hanging** 
     * since tokio::run blocks the current thread and waits for the entire runtime
     * to become idle but it will never becomes idle since we keep a reference
     * to a reqwest client which maintains a connection pool.
     */
    std::mem::drop(client);
    tokio::run(
        clip.join(clip2)
            .and_then(|(c1, c2)| {
                println!("{:?} {:?}", c1, c2);
                Ok((c1, c2))
            })
            .map(|_| ())
            .map_err(|_| ())
    );
}
