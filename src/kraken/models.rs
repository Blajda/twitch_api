extern crate serde_json;
extern crate chrono;

#[derive(Debug, Deserialize)]
pub struct Clip {
    pub slug: String,
    pub tracking_id: String,
    pub url: String,
    pub embed_url: String,
    pub embed_html: String,
    pub broadcaster: UserData,
    pub curator: UserData,
    pub vod: Vod,
    pub game: String,
    pub language: String,
    pub title: String,
    pub views: i32,
    pub duration: f32,
    pub created_at: String,
    pub thumbnails: Thumbnails,
}


#[derive(Debug, Deserialize)]
pub struct Thumbnails {
    pub medium: String,
    pub small: String,
    pub tiny: String,
}

#[derive(Debug, Deserialize)]
pub struct UserData {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub channel_url: String,
    pub logo: String,
}

#[derive(Debug, Deserialize)]
pub struct Vod {
    pub id: String,
    pub url: String,
}
