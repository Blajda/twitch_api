#![recursion_limit="128"]
#![feature(never_type)]
extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate chrono;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate twitch_types;

pub mod helix;
pub mod kraken;
pub mod types;
pub mod error;
mod sync;
pub mod namespace;
pub mod client;
pub mod models;

pub use self::helix::Client as HelixClient;
pub use self::kraken::Client as KrakenClient;
pub use self::client::{ClientConfig, TestConfig};
