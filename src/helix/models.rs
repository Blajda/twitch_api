extern crate serde_json;
extern crate chrono;

use url::Url;
use chrono::{DateTime, Utc};
use crate::types::{UserId, VideoId, ChannelId};

use crate::client::PaginationTrait;

#[derive(Debug, Deserialize, Serialize)]
pub struct DataContainer<T> {
    pub data: Vec<T>
}

impl<T> PaginationTrait for DataContainer<T> {
    fn cursor<'a>(&'a self) -> Option<&'a str> { None }
}

impl<T> PaginationTrait for PaginationContainer<T> {
    fn cursor<'a>(&'a self) -> Option<&'a str> { 
        match self.pagination.as_ref() {
            Some(cursor) => {
                match cursor.cursor.as_ref() {
                    Some(cursor) => Some(cursor),
                    None => None,
                }
            },
            None => None
        }
    }
}

impl PaginationTrait for Credentials {
    fn cursor<'a>(&'a self) -> Option<&'a str> { None }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationContainer<T> {
    pub data: Vec<T>,
    pub pagination: Option<Cursor>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Cursor {
    pub cursor: Option<String>
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
    pub broadcaster_id: ChannelId<'static>,
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
    pub scope: Option<Vec<String>>,
    pub token_type: String,
}
