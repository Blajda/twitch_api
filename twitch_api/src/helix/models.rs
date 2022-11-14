extern crate chrono;
extern crate serde_json;

use super::namespaces::IterableApiRequest;
use crate::client::{
    BidirectionalPagination, ForwardPagination, HelixPagination, PaginationContrainerTrait,
    RequestRef,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer};
use std::sync::Arc;
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
    fn next(
        &self,
    ) -> Option<super::namespaces::IterableApiRequest<PaginationContainer<T>, ApiError>> {
        match self.cursor() {
            Some(cursor) => Some(IterableApiRequest::from_request_with_cursor(
                self.base_request.as_ref().unwrap().clone(),
                Some(cursor.to_owned()),
                true,
            )),
            None => None,
        }
    }

    fn prev(
        &self,
    ) -> Option<super::namespaces::IterableApiRequest<PaginationContainer<T>, ApiError>> {
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
    pub stream_id: Option<VideoId<'static>>,
    pub user_id: UserId<'static>,
    pub user_login: String,
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
    pub duration: String,
    pub muted_segments: Vec<MuteSegment>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MuteSegment {
    pub duration: u32,
    pub offset: u32,
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
    pub duration: f32,
    pub vod_offset: Option<i32>,
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_clip_parse() {
        let data = r#"
        {
          "data": [
            {
              "id": "RandomClip1",
              "url": "https://clips.twitch.tv/AwkwardHelplessSalamanderSwiftRage",
              "embed_url": "https://clips.twitch.tv/embed?clip=RandomClip1",
              "broadcaster_id": "1234",
              "broadcaster_name": "JJ",
              "creator_id": "123456",
              "creator_name": "MrMarshall",
              "video_id": "",
              "game_id": "33103",
              "language": "en",
              "title": "random1",
              "view_count": 10,
              "created_at": "2017-11-30T22:34:18Z",
              "thumbnail_url": "https://clips-media-assets.twitch.tv/157589949-preview-480x272.jpg",
              "duration": 12.9,
              "vod_offset": 1957
            }
          ],
          "pagination": {
            "cursor": "eyJiIjpudWxsLCJhIjoiIn0"
          }
        }
        "#;

        let actual: PaginationContainer<Clip> = serde_json::from_str(data).unwrap();
    }

    #[test]
    pub fn test_video_parse() {
        let data = r#"
        {
            "data": [
              {
                "id": "335921245",
                "stream_id": null,
                "user_id": "141981764",
                "user_login": "twitchdev",
                "user_name": "TwitchDev",
                "title": "Twitch Developers 101",
                "description": "Welcome to Twitch development! Here is a quick overview of our products and information to help you get started.",
                "created_at": "2018-11-14T21:30:18Z",
                "published_at": "2018-11-14T22:04:30Z",
                "url": "https://www.twitch.tv/videos/335921245",
                "thumbnail_url": "https://static-cdn.jtvnw.net/cf_vods/d2nvs31859zcd8/twitchdev/335921245/ce0f3a7f-57a3-4152-bc06-0c6610189fb3/thumb/index-0000000000-%{width}x%{height}.jpg",
                "viewable": "public",
                "view_count": 1863062,
                "language": "en",
                "type": "upload",
                "duration": "3m21s",
                "muted_segments": [
                  {
                    "duration": 30,
                    "offset": 120
                  }
                ]
              }
            ],
            "pagination": {}
        }
        "#;
        let actual: PaginationContainer<Video> = serde_json::from_str(data).unwrap();
        assert_eq!(1, actual.data.len());
        let video = &actual.data[0];
        assert_eq!(video.duration, "3m21s");
        assert_eq!(1, video.muted_segments.len());
    }
}
