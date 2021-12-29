#![recursion_limit="128"]
#![feature(never_type)]
#![feature(into_future)]
extern crate serde;
extern crate chrono;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate log;
extern crate twitch_types;

pub mod helix;
pub mod error;
pub mod namespace;
pub mod client;
pub mod models;

pub use self::helix::Client as HelixClient;
pub use self::client::{ClientConfig};
