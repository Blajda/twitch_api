use futures::future::Future;
use std::sync::{Arc, Mutex};
use reqwest::r#async::Client as ReqwestClient;
pub use super::types;

use std::marker::PhantomData;

pub mod endpoints;
pub mod models;

use std::collections::HashSet;

use self::models::{DataContainer, PaginationContainer, Clip};

type EndPointResult<T> = Box<Future<Item=T, Error=reqwest::Error> + Send>;

pub trait UsersEndpoint {}
pub trait VideosEndpoint {}


pub trait ClipsEndpointTrait {
    fn clip(&self, id: &str) -> EndPointResult<DataContainer<Clip>>;
}

pub trait AuthEndpoint {}

pub struct Namespace<T> {
    client: Client,
    _type: PhantomData<T>
}

impl<T> Namespace<T> {
    pub fn new(client: &Client) -> Self {
        Namespace {
            client: client.clone(),
            _type: PhantomData,
        }
    }
}

pub struct Clips {}

type ClipsNamespace = Namespace<Clips>;

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientRef>,
}

struct MutClientRef {
    token: Option<String>,
    scopes: Vec<Scope>,
    previous: Option<Client>
}

struct ClientRef {
    id: String,
    client: ReqwestClient,
    inner: Mutex<MutClientRef>,
}

impl Client {
    pub fn new(id: &str) -> Client {
        let client = ReqwestClient::new();
        Client::new_with_client(id, client)
    }

    pub fn new_with_client(id: &str, client: ReqwestClient) -> Client {
        Client {
            inner: Arc::new(ClientRef {
                id: id.to_owned(),
                client: client,
                inner: Mutex::new(
                    MutClientRef {
                        token: None,
                        scopes: Vec::new(),
                        previous: None
                    })
            })
        }
    }
}


use reqwest::r#async::{RequestBuilder};
use reqwest::header;

impl Client {

    pub fn id(&self) -> &str {
        &self.inner.id
    }

    pub fn client(&self) -> &ReqwestClient {
        &self.inner.client
    }

    fn apply_standard_headers(&self, request: RequestBuilder) 
       -> RequestBuilder 
    {
        let client_header = header::HeaderValue::from_str(self.id()).unwrap();
        request.header("Client-ID", client_header)
    }
}

impl Client {

    pub fn clips(&self) -> ClipsNamespace {
        ClipsNamespace::new(self)
    }

}

impl ClipsNamespace {
    pub fn clip(self, id: &str) -> impl Future<Item=DataContainer<Clip>, Error=reqwest::Error> {
        use self::endpoints::clip;
        clip(self.client, id)
    }
}

pub struct AuthClientBuilder {
    scopes: HashSet<Scope>,
    secret: String,
    client: Client,
    /*If the user supplies a token,
    * then we can skip fetching it from the server and are authenticated
    */
    token: Option<String>,
}

impl AuthClientBuilder {
    pub fn new(client: Client, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder {
            scopes: HashSet::new(),
            secret: secret.to_owned(),
            client: client,
            token: None,
        }
    }

    /*TODO: Stack a new client ontop*/
    pub fn build(self) -> Client {
        self.client
    }

    pub fn scope(mut self, scope: Scope) -> AuthClientBuilder {
        let scopes = &mut self.scopes;
        scopes.insert(scope);
        self
    }

    pub fn scopes(mut self, scopes: Vec<Scope>) -> AuthClientBuilder {
        let _scopes = &mut self.scopes;
        for scope in scopes {
            _scopes.insert(scope);
        }
        self
    }

    pub fn token(mut self, token: &str) -> AuthClientBuilder {
        self.token.replace(token.to_owned());
        self
    }
}


#[derive(PartialEq, Hash, Eq)]
pub enum Scope {
    AnalyticsReadExtensions,
    AnalyticsReadGames,
    BitsRead,
    ClipsEdit,
    UserEdit,
    UserEditBroadcast,
    UserReadBroadcast,
    UserReadEmail,
}
