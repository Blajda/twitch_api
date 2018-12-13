use futures::Future;
use super::models::Clip;
use super::Client;

use super::API_DOMAIN;

impl Client {
    pub fn clip(&self, id: &str) 
        -> impl Future<Item=Clip, Error=reqwest::Error> 
    {
        let url = String::from("https://") + API_DOMAIN + "/kraken/clips/" + id;
        let client = self.create_reqwest_client();

        client
            .get(&url)
            .send()
            .map(|mut res| res.json::<Clip>())
            .and_then(|json| json)
    }
}
