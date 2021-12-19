use super::*;
use super::models::{DataContainer, User};
use std::string::ToString;

pub struct Users {}
type UsersNamespace = Namespace<Users>;

impl UsersNamespace {
    pub fn users<S: ToString>(self, ids: &[S], logins: &[S]) -> ApiRequest<DataContainer<User>> {
        users(self.client, ids, logins)
    }
}

impl Client {
    pub fn users(&self) -> UsersNamespace {
        UsersNamespace::new(self)
    }
}

/**
 * https://dev.twitch.tv/docs/api/reference#get-users
 */
pub fn users<S: ToString>(
        client: Client,
        ids: &[S],
        logins: &[S],
    ) -> ApiRequest<DataContainer<User>> {
        let client = client.inner;
        let url = client.api_base_uri().to_owned() + &String::from("/helix/users");

        let mut params: BTreeMap<&str, &dyn ToString> = BTreeMap::new();
        for id in ids {
            params.insert("id", id);
        }

        for login in logins {
            params.insert("login", login);
        }

        ApiRequest::new(url, params, client, Method::GET, Some(RatelimitKey::Default))
}

/**
 * Obtain the user using the Bearer Token
 * 
 * https://dev.twitch.tv/docs/api/reference#get-users
 */
pub fn authed_as(client: Client) -> ApiRequest<DataContainer<User>> {
    users(client, &[""], &[""])
}
