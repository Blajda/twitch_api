use std::error::Error;

use hyper::client::Client as HyperClient;
use hyper::{client::HttpConnector, Request};
use hyper::{Body, Method};
use hyper_tls::HttpsConnector;
use serde_derive::{Deserialize, Serialize};

pub struct MockClient {
    pub base_uri: String,
    pub hyper: HyperClient<HttpsConnector<HttpConnector>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientData {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Secret")]
    pub secret: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "IsExtension")]
    pub is_extension: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Container<T> {
    pub data: Vec<T>,
    pub cursor: String,
    pub total: u32,
}

impl MockClient {
    pub fn build() -> Self {
        let https = HttpsConnector::new();
        let hyper = HyperClient::builder().build::<_, Body>(https);

        MockClient {
            base_uri: "http://localhost:8080/units".to_string(),
            hyper,
        }
    }

    pub async fn clients(&self) -> Result<Container<ClientData>, Box<dyn Error>> {
        let r = Request::builder()
            .method(Method::GET)
            .uri(self.base_uri.to_string() + "/clients");
        let req = r.body(Body::empty())?;
        let res = self.hyper.request(req).await?;
        let (_parts, body) = res.into_parts();
        let bytes = hyper::body::to_bytes(body).await?;
        let res = serde_json::from_slice::<Container<ClientData>>(bytes.as_ref())?;

        Ok(res)
    }
}
