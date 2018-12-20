#![recursion_limit="128"]
#![feature(option_replace)]
extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate chrono;
#[macro_use] extern crate serde_derive;

use reqwest::r#async::Client as ReqwestClient;

pub mod helix;
pub mod kraken;
pub mod types;
pub mod error;
pub mod sync;
pub mod namespace;

pub use self::helix::Client as HelixClient;
pub use self::kraken::Client as KrakenClient;
