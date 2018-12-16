use futures::future::Future;
use super::super::models::{DataContainer, PaginationContainer, User, Video, Clip};
use super::super::Client; 
use std::collections::BTreeMap;
const API_DOMAIN: &'static str = "api.twitch.tv";
use super::super::Namespace;

pub struct Users {}
type UsersNamespace = Namespace<Users>;

impl UsersNamespace {
    pub fn users(self, id: Vec<&str>, login: Vec<&str>) -> impl Future<Item=DataContainer<User>, Error=reqwest::Error> {
        use self::users;
        users(self.client, id, login)
    }
}

impl Client {

    pub fn users(&self) -> UsersNamespace {
        UsersNamespace::new(self)
    }
}

pub fn users(
        client: Client,
        id: Vec<&str>,
        login: Vec<&str>,
    ) -> impl Future<Item = DataContainer<User>, Error = reqwest::Error> {
        let url =
            String::from("https://") + &String::from(API_DOMAIN) + &String::from("/helix/users");

        let mut params  = BTreeMap::new();
        for i in id {
            params.insert("id", i);
        }

        for log in login {
            params.insert("login", log);
        }

        let request = client.client().get(&url);
        let request = client.apply_standard_headers(request);
        let request = request.query(&params);

        request
            .send()
            .map(|mut res| {
                res.json::<DataContainer<User>>()
            })
            .and_then(|json| json)
}
