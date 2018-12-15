extern crate serde_json;
extern crate chrono;
extern crate url;

use url::Url;
use chrono::{DateTime, Utc};
use super::types::{UserId, VideoId};

#[derive(Debug, Deserialize)]
pub struct Clip {
    pub slug: String,
    pub tracking_id: String,
    #[serde(with = "url_serde")]
    pub url: Url,
    #[serde(with = "url_serde")]
    pub embed_url: Url,
    pub embed_html: String,
    pub broadcaster: UserData,
    pub curator: UserData,
    pub vod: Vod,
    pub game: String,
    pub language: String,
    pub title: String,
    pub views: i32,
    pub duration: f32,
    pub created_at: DateTime<Utc>,
    pub thumbnails: Thumbnails,
}


#[derive(Debug, Deserialize)]
pub struct Thumbnails {
    #[serde(with = "url_serde")]
    pub medium: Url,
    #[serde(with = "url_serde")]
    pub small: Url,
    #[serde(with = "url_serde")]
    pub tiny: Url,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub id: UserId,
    pub name: String,
    pub display_name: String,
    #[serde(with = "url_serde")]
    pub channel_url: Url,
    pub logo: String,
}

#[derive(Debug, Deserialize)]
pub struct Vod {
    pub id: VideoId,
    #[serde(with = "url_serde")]
    pub url: Url,
}
