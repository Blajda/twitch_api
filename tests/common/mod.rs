extern crate twitch_api;
extern crate hyper;
extern crate futures;
extern crate reqwest;
extern crate url;
extern crate http;

use tokio::runtime::current_thread::Runtime;
use twitch_api::{ClientConfig, TestConfig};
use reqwest::r#async::Response;
use http::response::Builder;

pub const CLIENT_ID: &str = "cfabdegwdoklmawdzdo98xt2fo512y";
pub const CLIENT_SECRET: &str = "nyo51xcdrerl8z9m56w9w6wg";

pub fn test_config() -> (ClientConfig, TestConfig) {
    let test_config = TestConfig::default();
    (ClientConfig {
        test_config: Some(test_config.clone()),
        max_retrys: 0,        
        ..ClientConfig::default()
    }, test_config)
}

pub fn okay_response(data: &'static str) -> Response {

    let response =
        Builder::new()
        .status(200)
        .body(data).unwrap();
    Response::from(response)
}