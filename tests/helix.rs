extern crate twitch_api;
extern crate tokio;
extern crate hyper;
extern crate futures;
extern crate url;
extern crate http;

pub mod common;

use futures::Stream;
use twitch_api::HelixClient;
use twitch_api::{ClientConfig, TestConfig};
use twitch_api::client::ClientTrait;
use twitch_api::error::Error;
use twitch_api::client::HelixScope;


use tokio::runtime::current_thread::Runtime;
use reqwest::r#async::Response;
use http::response::Builder;

use crate::common::*;

const USER_RESPONSE: &str = r#"{
        "data": [{
            "id": "44322889",
            "login": "dallas",
            "display_name": "dallas",
            "type": "staff",
            "broadcaster_type": "",
            "description": "Just a gamer playing games and chatting. :)",
            "profile_image_url": "https://static-cdn.jtvnw.net/jtv_user_pictures/dallas-profile_image-1a2c906ee2c35f12-300x300.png",
            "offline_image_url": "https://static-cdn.jtvnw.net/jtv_user_pictures/dallas-channel_offline_image-1a2c906ee2c35f12-1920x1080.png",
            "view_count": 191836881,
            "email": "login@provider.com"
        }]}"#;


const AUTH_RESPONSE: &str = r#"
{
  "access_token": "prau3ol6mg5glgek8m89ec2s9q5i3i",
  "refresh_token": "",
  "expires_in": 3600,
  "scope": ["user:read:email", "bits:read"],
  "token_type": "bearer"
}"#;

#[test]
fn test_invalid_client_id() {
    let response = r#"{"error":"Unauthorized","status":401,"message":"Must provide a valid Client-ID or OAuth token"}"#;

    let response = 
        Builder::new()
        .status(401)
        .body(response).unwrap();
    let response = Response::from(response);

    let (config, test_config) = test_config();
    test_config.push_response(response);

    let mut runtime = Runtime::new().unwrap();

    let client = HelixClient::new_with_config(CLIENT_ID, config);
    let user_future = client.users().users(&[], &["freakey"]);
    let result = runtime.block_on(user_future);

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.is_auth_error())
    }
}

#[test]
fn test_invalid_auth() {
    let response = r#"{"message":"invalid client secret","status":403}"#;

    let response = 
        Builder::new()
        .status(403)
        .body(response).unwrap();
    let response = Response::from(response);

    let (config, test_config) = test_config();
    test_config.push_response(response);
    let mut runtime = Runtime::new().unwrap();

    let client = HelixClient::new_with_config(CLIENT_ID, config)
        .authenticate(CLIENT_SECRET).build();
    assert!(!client.authenticated());

    let user_future = client.users().users(&[], &["dallas"]);
    let result = runtime.block_on(user_future);

    assert!(!client.authenticated());
    assert!(result.is_err());
    if let Err(err) = result {
        println!("{:?}", err);
        assert!(err.is_auth_error())
    }
}

#[test]
fn test_ratelimit_hit() {
    let response = r#"{"error":"Too Many Requests","message":"Thou Shall Not Pass","status":429}"#;
    let response = 
        Builder::new()
        .status(429)
        .body(response).unwrap();
    let response = Response::from(response);

    let (config, test_config) = test_config();
    test_config.push_response(response);
    let mut runtime = Runtime::new().unwrap();

    let client = HelixClient::new_with_config(CLIENT_ID, config);
    let user_future = client.users().users(&[], &["freakey"]);
    let result = runtime.block_on(user_future);

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.is_ratelimit_error())
    }
}

#[test]
fn test_client_header() {
    let (config, test_config) = test_config();
    test_config.push_response(okay_response(USER_RESPONSE));
    let mut runtime = Runtime::new().unwrap();

    let client = HelixClient::new_with_config(CLIENT_ID, config);
    let user_future = client.users().users(&[], &["dallas"]);
    let result = runtime.block_on(user_future);
    assert!(result.is_ok());

    let request = {
        let config = &mut test_config.inner.lock().unwrap();
        config.requests.pop().unwrap().unwrap()
    };

    let headers = request.headers();
    let header = headers.get("Client-Id");
    assert_eq!(Some(CLIENT_ID), header.and_then(|s| Some(s.to_str().unwrap())));
}

#[test]
fn test_auth_header() {

    let (config, test_config) = test_config();
    test_config.push_response(okay_response(USER_RESPONSE));
    test_config.push_response(okay_response(AUTH_RESPONSE));
    let mut runtime = Runtime::new().unwrap();

    let client = HelixClient::new_with_config(CLIENT_ID, config)
        .authenticate(CLIENT_SECRET).build();

    /*Authentication is lazy*/
    assert!(!client.authenticated());

    let user_future = client.users().users(&[], &["dallas"]);
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
    assert_eq!(Some("Bearer ".to_owned() + "prau3ol6mg5glgek8m89ec2s9q5i3i"), secret.and_then(|s| Some(s.to_str().unwrap().to_owned())));
    assert!(scopes.contains(&HelixScope::UserReadEmail));
    assert!(scopes.contains(&HelixScope::BitsRead));
}


#[test]
fn test_single_request() {
    let (config, test_config) = test_config();
    test_config.push_response(okay_response(USER_RESPONSE));
    let mut runtime = Runtime::new().unwrap();

    let client = HelixClient::new_with_config(CLIENT_ID, config);
    let user_future = client.users().users(&[], &["dallas"]);
    let result = runtime.block_on(user_future);

    assert!(result.is_ok())
}

#[test]
fn test_pagination() {

    let response = r#"{
        "data": [{
            "id": "234482848",
            "user_id": "67955580",
            "user_name": "ChewieMelodies",
            "title": "-",
            "description": "",
            "created_at": "2018-03-02T20:53:41Z",
            "published_at": "2018-03-02T20:53:41Z",
            "url": "https://www.twitch.tv/videos/234482848",
            "thumbnail_url": "https://static-cdn.jtvnw.net/s3_vods/bebc8cba2926d1967418_chewiemelodies_27786761696_805342775/thumb/thumb0-%{width}x%{height}.jpg",
            "viewable": "public",
            "view_count": 142,
            "language": "en",
            "type": "archive",
            "duration": "3h8m33s"
        }],
        "pagination":{"cursor":"eyJiIjpudWxsLCJhIjoiMTUwMzQ0MTc3NjQyNDQyMjAwMCJ9"}
        }"#;

    let response2 = r#"{
        "data": [{
            "id": "234482848",
            "user_id": "67955580",
            "user_name": "ChewieMelodies",
            "title": "-",
            "description": "",
            "created_at": "2018-03-02T20:53:41Z",
            "published_at": "2018-03-02T20:53:41Z",
            "url": "https://www.twitch.tv/videos/234482848",
            "thumbnail_url": "https://static-cdn.jtvnw.net/s3_vods/bebc8cba2926d1967418_chewiemelodies_27786761696_805342775/thumb/thumb0-%{width}x%{height}.jpg",
            "viewable": "public",
            "view_count": 142,
            "language": "en",
            "type": "archive",
            "duration": "3h8m33s"
        }],
        "pagination":{"cursor":""}
        }"#;

    let (config, test_config) = test_config();
    test_config.push_response(okay_response(response));
    test_config.push_response(okay_response(response2));
    let mut runtime = Runtime::new().unwrap();

    let client = HelixClient::new_with_config(CLIENT_ID, config);
    let video_future = client.videos().by_user(&"67955580");
    let result = runtime.block_on(video_future.into_future());
    assert!(result.is_ok());
    if let Ok((Some(data), next)) = result {

        let result = runtime.block_on(next.into_future());
        assert!(result.is_ok());
    } else {unreachable!()}
}