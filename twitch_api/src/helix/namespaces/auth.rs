use crate::client::RequestBuilder;

use super::models::Credentials;
use super::*;

pub struct Auth {}
type AuthNamespace = Namespace<Auth>;

impl AuthNamespace {
    pub fn client_credentials(self, secret: &str) -> RequestBuilder<Credentials> {
        client_credentials(self.client, &secret)
    }
}

impl Client {
    pub fn auth(&self) -> AuthNamespace {
        AuthNamespace::new(self)
    }
}

/**
 * https://dev.twitch.tv/docs/authentication/getting-tokens-oauth/#oauth-client-credentials-flow
*/
pub fn client_credentials<S: ToString>(client: Client, secret: &S) -> RequestBuilder<Credentials> {
    //TODO: Implement scopes
    let client = client.inner;
    let url = client.auth_base_uri().to_owned() + "/oauth2/token";
    let client_id = client.id();

    let mut b = RequestBuilder::new(client.clone(), url, Method::POST);

    b = b
        .with_query("client_id", &client_id)
        .with_query("client_secret", secret)
        .with_query("grant_type", "client_credentials")
        .with_query("scope", &"");

    return b;
}
