extern crate twitch_api;
extern crate tokio;
extern crate hyper;
extern crate futures;
extern crate url;
extern crate http;

pub mod common;

use twitch_api::KrakenClient;
use twitch_api::{ClientConfig, TestConfig};
use twitch_api::client::ClientTrait;
use twitch_api::error::Error;
use twitch_api::client::KrakenScope;

use tokio::runtime::current_thread::Runtime;
use reqwest::r#async::Response;
use http::response::Builder;

use crate::common::*;

const AUTH_RESPONSE: &str = r#"
{
  "access_token": "prau3ol6mg5glgek8m89ec2s9q5i3i",
  "refresh_token": "",
  "expires_in": 3600,
  "scope": ["user_read", "channel_commercial"],
  "token_type": "bearer"
}"#;

const USER_RESPONSE: &str = r#"{
    "_id": "44322889",
    "bio": "Just a gamer playing games and chatting. :)",
    "created_at": "2013-06-03T19:12:02.580593Z",
    "display_name": "dallas",
    "logo": "https://static-cdn.jtvnw.net/jtv_user_pictures/dallas-profile_image-1a2c906ee2c35f12-300x300.png",
    "name": "dallas",
    "type": "staff",
    "updated_at": "2016-12-13T16:31:55.958584Z"
}"#;

#[test]
fn test_auth_header() {

    let (config, test_config) = test_config();
    test_config.push_response(okay_response(USER_RESPONSE));
    test_config.push_response(okay_response(AUTH_RESPONSE));
    let mut runtime = Runtime::new().unwrap();

    let client = KrakenClient::new_with_config(CLIENT_ID, config)
        .authenticate(CLIENT_SECRET).build();

    /*Authentication is lazy*/
    assert!(!client.authenticated());

    let user_future = client.users().by_id(&"44322889");
    let result = runtime.block_on(user_future);
    assert!(result.is_ok());

    let request = {
        let config = &mut test_config.inner.lock().unwrap();
        config.requests.pop().unwrap().unwrap()
    };

    let headers = request.headers();
    let client_id = headers.get("Client-Id");
    let secret = headers.get("Authorization");

    let scopes = client.scopes();

    assert!(client.authenticated());
    assert_eq!(Some(CLIENT_ID), client_id.and_then(|s| Some(s.to_str().unwrap())));
    assert_eq!(Some("OAuth ".to_owned() + "prau3ol6mg5glgek8m89ec2s9q5i3i"), secret.and_then(|s| Some(s.to_str().unwrap().to_owned())));
    assert!(scopes.contains(&KrakenScope::UserRead));
    assert!(scopes.contains(&KrakenScope::ChannelCommercial));
}