extern crate futures;
extern crate reqwest;
extern crate serde;
extern crate chrono;

#[macro_use]
extern crate serde_derive;

mod helix;
mod kraken;

pub use self::helix::endpoints::Client as HelixClient;
pub use self::kraken::Client as KrakenClient;

pub struct Client {
    pub helix: HelixClient,
    pub kraken: KrakenClient,
}

impl Client {
    pub fn new(client_id: &str) -> Client {
        Client {
            helix:  HelixClient::new(client_id),
            kraken: KrakenClient::new(client_id),
        }
    }
}
