use super::*;
use super::models::{DataContainer, User};
use crate::types::UserId;

pub struct Users {}
type UsersNamespace = Namespace<Users>;

impl UsersNamespace {
    pub fn users(self, ids: Vec<&UserId>, logins: Vec<&str>) 
        -> ApiRequest<DataContainer<User>> {
        use self::users;
        users(self.client, ids, logins)
    }
}

impl Client {
    pub fn users(&self) -> UsersNamespace {
        UsersNamespace::new(self)
    }
}

pub fn users(
        client: Client,
        ids: Vec<&UserId>,
        logins: Vec<&str>,
    ) -> ApiRequest<DataContainer<User>> {
        let client = client.inner;
        let url =
            String::from("https://") + client.domain() + &String::from("/helix/users");

        let mut params  = BTreeMap::new();
        for id in ids {
            params.insert("id", id.as_ref());
        }

        for login in logins {
            params.insert("login", login);
        }

        ApiRequest::new(url, params, client, Method::GET, Some(RatelimitKey::Default))
}

pub fn authed_as(client: Client) -> ApiRequest<DataContainer<User>> {
    users(client, Vec::with_capacity(0), Vec::with_capacity(0))
}
