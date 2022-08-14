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
use tokio::time::{Duration, Instant};
use twitch_api::HelixClient;

use twitch_api::client::BidirectionalPagination;

fn report_percentiles(counts: &[u128]) {
    let items = counts.len();
    let p999 = counts[items * 999 / 1000];
    let p99 = counts[items * 99 / 100];
    let p98 = counts[items * 98 / 100];
    let p95 = counts[items * 95 / 100];
    let p75 = counts[items * 3 / 4];
    let p50 = counts[items / 2];
    let p25 = counts[items / 4];
    let p0 = counts[0];

    println!("p100: {}", counts[items - 1]);
    println!("p99.9: {}", p999);
    println!("p99: {}", p99);
    println!("p98: {}", p98);
    println!("p95: {}", p95);
    println!("p75: {}", p75);
    println!("p50: {}", p50);
    println!("p25: {}", p25);
    println!("p0: {}", p0);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv();
    env_logger::init();

    let client_id = &env::var("TWITCH_API")?;
    let client_secret = &env::var("TWITCH_SECRET")?;

    let helix_client = HelixClient::new(client_id)
        .authenticate(client_secret)
        .build()
        .await?;

    let scrape_start = Instant::now();
    let mut request = Some(helix_client.streams().get().first(100).build_iterable());
    let mut views = Vec::new();
    let mut request_times = Vec::new();

    while let Some(r) = request {
        let start = Instant::now();
        let page = r.await?;
        request = page.next();
        let duration = Instant::now().checked_duration_since(start).unwrap();
        request_times.push(duration.as_micros());

        for stream in &page.data {
            views.push(stream.viewer_count as u128);
        }
    }

    let scrape_end = Instant::now();
    views.sort();
    request_times.sort();

    println!(
        "It took {}s to scrape all streams",
        scrape_end
            .checked_duration_since(scrape_start)
            .unwrap()
            .as_secs()
    );
    println!("There are {} active streams", views.len());
    println!("These are the percentiles of views");
    report_percentiles(&views);

    println!("");
    println!("{} api requests were made", request_times.len());
    println!("These are the percentiles of api request times (mircoseconds)");
    report_percentiles(&request_times);

    return Ok(());
}
