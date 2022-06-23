extern crate chrono;
extern crate serde_json;

use super::namespaces::IterableApiRequest;
use crate::client::{
    BidirectionalPagination, ForwardPagination, HelixPagination, PaginationContrainerTrait,
    RequestRef,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use std::{sync::Arc};
use twitch_types::{BroadcasterId, GameId, StreamId, UserId, VideoId};
use url::Url;

fn null_as_empty<'de, D, T>(de: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let key = Option::deserialize(de)?;
    match key {
        Some(list) => Ok(list),
        None => Ok(Vec::new()),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DataContainer<T> {
    pub data: Vec<T>,
}

impl<T> ForwardPagination for DataContainer<T> {
    fn cursor<'a>(&'a self) -> Option<&'a str> {
        None
    }
}

impl<T> ForwardPagination for PaginationContainer<T> {
    fn cursor<'a>(&'a self) -> Option<&'a str> {
        match self.pagination.as_ref() {
            Some(cursor) => match cursor.cursor.as_ref() {
                Some(cursor) => Some(cursor),
                None => None,
            },
            None => None,
        }
    }
}

impl<T> HelixPagination for PaginationContainer<T> {}

impl<T> BidirectionalPagination<PaginationContainer<T>, ApiError> for PaginationContainer<T> {
    fn next(&self) -> Option<super::namespaces::IterableApiRequest<PaginationContainer<T>, ApiError>> {
        match self.cursor() {
            Some(cursor) => Some(IterableApiRequest::from_request_with_cursor(
                self.base_request.as_ref().unwrap().clone(),
                Some(cursor.to_owned()),
                true,
            )),
            None => None,
        }
    }

    fn prev(&self) -> Option<super::namespaces::IterableApiRequest<PaginationContainer<T>, ApiError>> {
        match self.cursor() {
            Some(cursor) => Some(IterableApiRequest::from_request_with_cursor(
                self.base_request.as_ref().unwrap().clone(),
                Some(cursor.to_owned()),
                false,
            )),
            None => None,
        }
    }
}

impl ForwardPagination for Credentials {
    fn cursor<'a>(&'a self) -> Option<&'a str> {
        None
    }
}

impl<T> PaginationContrainerTrait for PaginationContainer<T> {
    fn set_last_direction(&mut self, forward: bool) {
        self.last_direction = Some(forward);
    }

    fn set_last_cursor(&mut self, cursor: String) {
        self.last_cursor = Some(cursor);
    }

    fn set_base_request(&mut self, request: std::sync::Arc<RequestRef>) {
        self.base_request = Some(request);
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationContainer<T> {
    pub data: Vec<T>,
    pub pagination: Option<Cursor>,

    #[serde(skip)]
    last_cursor: Option<String>,
    #[serde(skip)]
    last_direction: Option<bool>,
    #[serde(skip)]
    base_request: Option<Arc<RequestRef>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cursor {
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Video {
    pub id: VideoId<'static>,
    pub user_id: UserId<'static>,
    pub user_name: String,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub published_at: DateTime<Utc>,
    #[serde(with = "url_serde")]
    pub url: Url,
    /*FIXME: Serde will attempt to parse an empty string.
     * In this case this should be None when thumbnail_url is an empty string
     */
    //#[serde(with = "url_serde")]
    pub thumbnail_url: String, //Option<Url>,
    pub viewable: String,
    pub view_count: i32,
    pub language: String,
    #[serde(rename = "type")]
    pub video_type: String,
    //Should be converted to a Duration
    pub duration: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: UserId<'static>,
    pub login: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub broadcaster_type: String,
    pub description: String,
    #[serde(with = "url_serde")]
    pub profile_image_url: Url,
    //#[serde(with = "url_serde")]
    pub offline_image_url: String, // Option<Url>,
    pub view_count: u32,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Clip {
    pub id: String,
    #[serde(with = "url_serde")]
    pub url: Url,
    #[serde(with = "url_serde")]
    pub embed_url: Url,
    pub broadcaster_id: BroadcasterId<'static>,
    pub broadcaster_name: String,
    pub creator_id: UserId<'static>,
    pub creator_name: String,
    pub video_id: VideoId<'static>,
    pub game_id: String,
    pub language: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    #[serde(with = "url_serde")]
    pub thumbnail_url: Url,
    pub view_count: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
    #[serde(default)]
    #[serde(deserialize_with = "null_as_empty")]
    pub scope: Vec<String>,
    pub token_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Channel {
    pub broadcaster_id: BroadcasterId<'static>,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    pub broadcaster_language: String,
    pub game_id: GameId<'static>,
    pub game_name: String,
    pub title: String,
    pub delay: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Stream {
    pub id: StreamId<'static>,
    pub user_id: UserId<'static>,
    pub user_login: String,
    pub user_name: String,
    pub game_id: GameId<'static>,
    pub game_name: String,
    #[serde(rename = "type")]
    pub stream_type: String,
    pub title: String,
    pub viewer_count: u32,
    pub started_at: String,
    pub language: String,
    pub thumbnail_url: String,
    #[serde(deserialize_with = "null_as_empty")]
    pub tag_ids: Vec<String>,
    pub is_mature: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ApiError {
    pub error: String,
    pub status: u32,
    pub message: String,
}
