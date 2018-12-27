extern crate dotenv;
extern crate futures;
extern crate serde;
extern crate tokio;
extern crate twitch_api;

use futures::future::Future;
use futures::Stream;
use std::env;
use twitch_api::HelixClient;

fn main() {
    dotenv::dotenv().unwrap();
    let client_id = &env::var("TWITCH_API").unwrap();
    let client =  HelixClient::new(client_id);


    let authed_client = 
        client
        .authenticate(&env::var("TWITCH_SECRET").unwrap())
        .build();

    let clip = authed_client
        .clips()
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });

    let clip2 = authed_client
        .clips()
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });

    let videos = authed_client
        .videos()
        .by_user("67955580")
        .take(10)
        .for_each(|collection| {
            println!("{:?}", collection);
            Ok(())
        })
        .map(|_| ())
        .map_err(|err| {println!("{:?}", err); ()});


    /*
    let clip2 = client.kraken
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });
        */

    /* Prevents tokio from **hanging** 
     * since tokio::run blocks the current thread and waits for the entire runtime
     * to become idle but it will never becomes idle since we keep a reference
     * to a reqwest client which maintains a connection pool.
     */
    //std::mem::drop(authed_client);
    tokio::run(
        /*
        clip.join(clip2)
            .and_then(|(c1, c2)| {
                println!("{:?} {:?}", c1, c2);
                Ok((c1, c2))
            }).and_then(move |_| {
                authed_client
                    .clips()
                    .clip(&"EnergeticApatheticTarsierThisIsSparta")
                    .map(|_| ())
                    .map_err(|err| {
                        println!("{:?}", err); 
                        ()
                    })
            })
            .map(|_| ())
            .map_err(|_| ())
            */
        videos
    );
}
