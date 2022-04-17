use crate::client::RequestBuilder;

use super::models::{DataContainer, User};
use super::*;
use std::string::ToString;

pub struct Users {}
type UsersNamespace = Namespace<Users>;

impl UsersNamespace {
    pub fn users<S: ToString>(
        self,
        ids: &[S],
        logins: &[S],
    ) -> RequestBuilder<DataContainer<User>> {
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
) -> RequestBuilder<DataContainer<User>> {
    let client = client.inner;
    let url = client.api_base_uri().to_owned() + &String::from("/helix/users");
    let mut b = RequestBuilder::new(client, url, Method::GET);

    /*TODO: This doesn't support a list of userids and clients... */
    for id in ids {
        b = b.with_query("id", id);
    }

    for login in logins {
        b = b.with_query("login", login);
    }

    return b;
}

/**
 * Obtain the user using the Bearer Token
 *
 * https://dev.twitch.tv/docs/api/reference#get-users
 */
pub fn authed_as(client: Client) -> RequestBuilder<DataContainer<User>> {
    users(client, &[""], &[""])
}
