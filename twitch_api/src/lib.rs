#![recursion_limit = "128"]
#![feature(never_type)]
#![feature(into_future)]
#![feature(backtrace)]
extern crate chrono;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
extern crate twitch_types;

pub mod client;
pub mod error;
pub mod helix;
pub mod models;
pub mod namespace;

pub use self::client::ClientConfig;
pub use self::helix::Client as HelixClient;
