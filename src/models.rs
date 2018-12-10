extern crate serde_json;

#[derive(Debug, Deserialize)]
pub struct DataContainer<T> {
    pub data: Vec<T>
}

#[derive(Debug, Deserialize)]
pub struct Cursor {
    cursor: String
}

#[derive(Debug, Deserialize)]
pub struct PaginationContainer<T> {
    pub data: Vec<T>,
    pub pagination: Option<Cursor>
}

#[derive(Debug, Deserialize)]
pub struct Video {
    pub id: String,
    pub user_id: String,
    pub user_name: String,
    pub title: String,
    pub description: String,
    //Should be converted to a DateTime
    pub created_at: String,
    pub published_at: String,
    //Should be converted to a URL
    pub url: String,
    thumbnail_url: String,
    viewable: String,
    view_count: i32,
    language: String,
    #[serde(rename = "type")]
    video_type: String,
    //Should be converted to a Duration
    duration: String,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub id: String,
    pub login: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: u32,
    pub email: Option<String>,
}
