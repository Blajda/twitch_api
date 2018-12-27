#![recursion_limit="128"]
#![feature(option_replace)]
extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate chrono;
#[macro_use] extern crate serde_derive;

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
