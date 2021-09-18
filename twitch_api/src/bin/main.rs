extern crate dotenv;
extern crate futures;
extern crate serde;
extern crate tokio;
extern crate twitch_api;
extern crate env_logger;

use futures::future::Future;
use std::env;
use twitch_api::HelixClient;
use twitch_api::KrakenClient;
use twitch_api::ClientConfig;
use twitch_api::client::RatelimitMap;
use twitch_types::UserId;


fn main() {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let config = ClientConfig {
        max_retrys: 0,
        ratelimits: RatelimitMap::default(),
        ..ClientConfig::default()
    };

    let client_id = &env::var("TWITCH_API").unwrap();
    let helix_client =  HelixClient::new_with_config(client_id, config);
    let _kraken_client = KrakenClient::new(client_id);

        /*
        .authenticate(&env::var("TWITCH_SECRET").unwrap())
        .build();
        */

/*
    let clip = helix_client
        .clips()
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });
        */
    /*

    let clip2 = authed_client
        .clips()
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });
    */

    //use twitch_api::types::VideoId;

    /*
    let videos = authed_client
        .videos()
        .by_user(&UserId::from_str("19571641").unwrap())
        .take(1)
        .for_each(|collection| {
            println!("{:?}", collection);
            Ok(())
        })
        .map(|_| ())
        .map_err(|err| {println!("{:?}", err); ()});
        */


/*
    let clip2 = kraken_client
        .clips()
        .clip(&"EnergeticApatheticTarsierThisIsSparta")
        .map_err(|err| {
            println!("{:?}", err); 
            ()
        });
*/


    let f = futures::future::ok(1).and_then(move |_| {
        let id = UserId::from_str("1").unwrap();
        for _i in 0..80 {
            let u = helix_client
                .users()
                .users(&vec!(id.as_ref()), &vec!("freakey"))
                .map(|res| {println!("{:?}", res); ()})
                .map_err(|res| {println!("{:?}", res); ()});
                tokio::spawn(u);
        }
        Ok(())
    });

    /* Prevents tokio from **hanging** 
     * since tokio::run blocks the current thread and waits for the entire runtime
     * to become idle but it will never becomes idle since we keep a reference
     * to a reqwest client which maintains a connection pool.
     */
    //std::mem::drop(authed_client);
    tokio::run(
        f
        /*
        clip.join(clip2)
            .and_then(|(c1, c2)| {
                println!("{:?}", c1);
                println!("__");
                println!("{:?}", c2);
                Ok((c1, c2))
            }).and_then(move |_| {
                helix_client
                    .clips()
                    .clip(&ClipId::new("EnergeticApatheticTarsierThisIsSparta"))
                    .map(|_| ())
                    .map_err(|err| {
                        println!("{:?}", err); 
                        ()
                    })
            })
            .map(|_| ())
            .map_err(|_| ())
            */
        /*videos*/
    );
}
