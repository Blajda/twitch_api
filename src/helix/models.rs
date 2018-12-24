extern crate serde_json;
extern crate chrono;

use url::Url;
use chrono::{DateTime, Utc};
use super::types::{UserId, VideoId, ChannelId};

pub trait PaginationTrait {
    fn cursor<'a>(&'a self) -> &'a Option<Cursor>;
    fn set_request(&mut self);
}

#[derive(Debug, Deserialize)]
pub struct DataContainer<T> {
    pub data: Vec<T>
}

impl<T> PaginationTrait for DataContainer<T> {
    fn cursor<'a>(&'a self) -> &'a Option<Cursor> { &None }
    fn set_request(&mut self) {}
}

impl<T> PaginationTrait for PaginationContainer<T> {
    fn cursor<'a>(&'a self) -> &'a Option<Cursor> { &self.pagination }
    fn set_request(&mut self) {}
}

impl PaginationTrait for Credentials {
    fn cursor<'a>(&'a self) -> &'a Option<Cursor> { &None }
    fn set_request(&mut self) {}
}

#[derive(Debug, Deserialize)]
pub struct PaginationContainer<T> {
    pub data: Vec<T>,
    pub pagination: Option<Cursor>
}

#[derive(Debug, Deserialize)]
pub struct Cursor {
    cursor: String
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
    pub scope: Option<Vec<String>>,
    pub token_type: String,
}


#[derive(Debug, Deserialize)]
pub struct Video {
    pub id: VideoId,
    pub user_id: UserId,
    pub user_name: String,
    pub title: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub published_at: DateTime<Utc>,
    #[serde(with = "url_serde")]
    pub url: Url,
    #[serde(with = "url_serde")]
    pub thumbnail_url: Url,
    pub viewable: String,
    pub view_count: i32,
    pub language: String,
    #[serde(rename = "type")]
    pub video_type: String,
    //Should be converted to a Duration
    pub duration: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: UserId,
    pub login: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub broadcaster_type: String,
    pub description: String,
    #[serde(with = "url_serde")]
    pub profile_image_url: Url,
    #[serde(with = "url_serde")]
    pub offline_image_url: Url,
    pub view_count: u32,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Clip {
    pub id: String,
    #[serde(with = "url_serde")]
    pub url: Url,
    #[serde(with = "url_serde")]
    pub embed_url: Url,
    pub broadcaster_id: ChannelId,
    pub broadcaster_name: String,
    pub creator_id: UserId,
    pub creator_name: String,
    pub video_id: VideoId,
    pub game_id: String,
    pub language: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    #[serde(with = "url_serde")]
    pub thumbnail_url: Url,
    pub view_count: i32,
}
