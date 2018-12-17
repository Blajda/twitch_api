use futures::future::Future;
use std::sync::{Arc, Mutex};
use reqwest::r#async::Client as ReqwestClient;
pub use super::types;

use std::marker::PhantomData;
pub mod models;
pub mod namespaces;

use std::collections::HashSet;

use self::models::{DataContainer, PaginationContainer, Clip};
use futures::{Sink, Stream};

type EndPointResult<T> = Box<Future<Item=T, Error=reqwest::Error> + Send>;

pub trait UsersEndpoint {}
pub trait VideosEndpoint {}


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

#[derive(PartialEq, Hash, Eq, Clone)]
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

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientRef>,
}

use reqwest::r#async::Response;
use futures::sync::oneshot;

struct MutClientRef {
    token: Option<String>,
    scopes: Vec<Scope>,
    previous: Option<Client>,
    chan: Option<mpsc::Sender<(Arc<RequestRef>, oneshot::Sender<Response>)>>,
}

use futures::sync::mpsc;


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
                        chan: None,
                        token: None,
                        scopes: Vec::new(),
                        previous: None
                    })
            })
        }
    }

    pub fn id(&self) -> &str {
        &self.inner.id
    }

    pub fn client(&self) -> &ReqwestClient {
        &self.inner.client
    }

    pub fn authenticated(&self) -> bool {
        let mut_data = self.inner.inner.lock().unwrap();
        mut_data.token.is_some()
    }

    /*
    pub fn scopes(&self) -> Vec<Scope> {
        let mut_data = self.inner.inner.lock().unwrap();
        (&mut_data.scopes).into_iter().to_owned().collect()
    }
    */

    pub fn authenticate(self) -> AuthClientBuilder {
        AuthClientBuilder::new(self)
    }

    pub fn deauthenticate(self) -> Client {
        let mut_data = self.inner.inner.lock().unwrap();
        match &mut_data.previous {
            Some(old_client) => old_client.clone(),
            None => self.clone()
        }
    }

    pub fn apply_standard_headers(&self, request: RequestBuilder) 
       -> RequestBuilder 
    {
        let client_header = header::HeaderValue::from_str(self.id()).unwrap();
        request.header("Client-ID", client_header)
    }
}


use reqwest::r#async::{RequestBuilder};
use reqwest::header;


pub struct AuthClientBuilder {
    scopes: HashSet<Scope>,
    secret: Option<String>,
    token: Option<String>,
    client: Client,
    /*If the user supplies a token,
    * then we can skip fetching it from the server and are authenticated
    */
}

impl AuthClientBuilder {
    pub fn new(client: Client) -> AuthClientBuilder {
        AuthClientBuilder {
            scopes: HashSet::new(),
            client: client,
            secret: None,
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

use std::collections::BTreeMap;

struct RequestRef {
    url: String,
    params: BTreeMap<String, String>,
    client: Client,

}

enum RequestState<T> {
    Uninitalized,
    PollChannel(oneshot::Receiver<Response>),
    PollParse(Box<dyn Future<Item=T, Error=reqwest::Error> + Send>),
}

pub struct ApiRequest<T> {
    inner: Arc<RequestRef>,
    state: RequestState<T>
}

impl<T: DeserializeOwned + 'static + Send> ApiRequest<T> {

    pub fn new(url: String,
               params: BTreeMap<String, String>,
               client: Client
               ) -> ApiRequest<T>
    {
        ApiRequest {
            inner: Arc::new( RequestRef {
                url: url,
                params: params,
                client: client,
            }),
            state: RequestState::Uninitalized
        }
    }
}


use futures::Poll;
use serde::de::DeserializeOwned;
use futures::Async;
use futures::try_ready;

fn handle_requests(channel: mpsc::Receiver<(Arc<RequestRef>, oneshot::Sender<Response>)>) 
    -> impl Future<Item=(), Error=()>
{
    channel.for_each(|(request, notify)| {
        let _request = request.client.client().get(&request.url);
        let _request = request.client.apply_standard_headers(_request);
        let _request = _request.query(&request.params);

        let f = _request
            .send()
            .map(move |response| {
                notify.send(response);
                ()
            }).
            map_err(|_| {
                panic!("TODO....")
            });

        tokio::spawn(f);
        
        Ok(())
    })
    .map(|_| ())
    .map_err(|_| ())
}

impl<T: DeserializeOwned + 'static + Send> Future for ApiRequest<T> {
    type Item = T;
    type Error = reqwest::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match &mut self.state {
                RequestState::Uninitalized => {
                    /*TODO use poll_ready*/
                    let mut mut_client = self.inner.client.inner.inner.lock().unwrap();
                    let (resp_tx, resp_rx) = oneshot::channel();
                    match &mut mut_client.chan {
                        Some(chan) => {
                            chan.try_send((self.inner.clone(), resp_tx));
                        },
                        None => {
                            let (mut chan_tx, chan_rx) = mpsc::channel(30);
                            chan_tx.try_send((self.inner.clone(), resp_tx));

                            tokio::spawn(handle_requests(chan_rx));
                            mut_client.chan.replace(chan_tx);
                        }
                    }

                    self.state = RequestState::PollChannel(resp_rx);
                },
                RequestState::PollChannel(chan) => {
                    let status = chan.poll();
                    match status {
                        Ok(Async::NotReady) => return Ok(Async::NotReady),
                        Ok(Async::Ready(mut res)) => {
                            let f = res.json::<T>();
                            self.state = RequestState::PollParse(Box::new(f));
                            continue;
                        },
                        _ => panic!("TODO...")
                    }
                },
                RequestState::PollParse(future) => {
                    let res = try_ready!(future.poll());
                    return Ok(Async::Ready(res));
                }
            }
        }
    }
}
