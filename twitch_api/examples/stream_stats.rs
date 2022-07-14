//! Paginate through all active streams and determine percentiles for viewership
//!
//! Demonstrates how to use the library's pagination traits to obtain additonal
//! results

extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate tokio;
extern crate twitch_api;

use std::env;
use std::error::Error;
use twitch_api::HelixClient;

use twitch_api::client::BidirectionalPagination;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().unwrap();
    env_logger::init();

    let client_id = &env::var("TWITCH_API")?;
    let client_secret = &env::var("TWITCH_SECRET")?;

    let helix_client = HelixClient::new(client_id)
        .authenticate(client_secret)
        .build()
        .await?;

    let mut request = Some(helix_client.streams().get().first(100).build_iterable());
    let mut views = Vec::new();

    while let Some(r) = request {
        let page = r.await?;
        request = page.next();

        for stream in &page.data {
            views.push(stream.viewer_count);
        }
    }

    views.sort();
    let total_streams = views.len();
    let p999 = views[total_streams * 999 / 1000];
    let p99 = views[total_streams * 99 / 100];
    let p98 = views[total_streams * 98 / 100];
    let p95 = views[total_streams * 95 / 100];
    let p75 = views[total_streams * 3 / 4];
    let p50 = views[total_streams / 2];

    println!("There are {} active streams", total_streams);
    println!("These are the percentiles of views");
    println!("p100: {}", views[total_streams - 1]);
    println!("p99.9: {}", p999);
    println!("p99: {}", p99);
    println!("p98: {}", p98);
    println!("p95: {}", p95);
    println!("p75: {}", p75);
    println!("p50: {}", p50);

    return Ok(());
}
