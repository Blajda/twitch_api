use futures::future::Future;
use std::sync::{Arc, Mutex};
use reqwest::r#async::Client as ReqwestClient;
pub use super::types;

use std::marker::PhantomData;
pub mod models;
pub mod namespaces;

use std::collections::HashSet;
use futures::{Sink, Stream};

use super::error::Error;

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

#[derive(Clone, PartialEq)]
enum AuthState {
    Unauth,
    Auth,
}

struct MutClientRef {
    token: Option<String>,
    scopes: Vec<Scope>,
    previous: Option<Client>,
    auth_barrier: Barrier<AuthWaiter>,
    auth_state: AuthState,
}

use futures::sync::mpsc;


struct ClientRef {
    id: String,
    secret: Option<String>,
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
                secret: None,
                inner: Mutex::new(
                    MutClientRef {
                        auth_barrier: Barrier::new(),
                        token: None,
                        scopes: Vec::new(),
                        previous: None,
                        auth_state: AuthState::Auth,
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

    /* The 'bottom' client must always be a client that is not authorized.
     * This which allows for calls to Auth endpoints using the same control flow
     * as other requests.
     *
     * Clients created with 'new' are bottom clients and calls
     * to authenticate stack a authed client on top
     */
    fn get_bottom_client(&self) -> Client {
        let mut_client = self.inner.inner.lock().unwrap();
        match &mut_client.previous {
            Some(client) => {
                client.get_bottom_client()
            },
            None => {
                self.clone()
            }
        }
    }

    pub fn authenticate(self, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder::new(self, secret)
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
        let mut_client = self.inner.inner.lock().unwrap();
        let client_header = header::HeaderValue::from_str(self.id()).unwrap();

        let request =
            if let Some(token) = &mut_client.token {
                let value = "Bearer ".to_owned() + token;
                let token_header = header::HeaderValue::from_str(&value).unwrap();
                request.header("Authorization", token_header)
            } else { request };

        request.header("Client-ID", client_header)
    }
}


use reqwest::r#async::{RequestBuilder};
use reqwest::header;


pub struct AuthClientBuilder {
    scopes: HashSet<Scope>,
    secret: String,
    token: Option<String>,
    client: Client,
    /*If the user supplies a token,
    * then we can skip fetching it from the server and are authenticated
    */
}

impl AuthClientBuilder {
    pub fn new(client: Client, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder {
            scopes: HashSet::new(),
            client: client,
            secret: secret.to_owned(),
            token: None,
        }
    }

    pub fn build(self) -> Client {
        let auth_state = if self.token.is_some() { AuthState::Auth } else { AuthState::Unauth };
        let old_client = self.client;
        Client {
            inner: Arc::new(ClientRef {
                id: old_client.inner.id.clone(),
                client: old_client.inner.client.clone(),
                secret: Some(self.secret),
                inner: Mutex::new (
                    MutClientRef {
                        auth_barrier: Barrier::new(),
                        token: self.token,
                        scopes: Vec::new(),
                        previous: Some(old_client),
                        auth_state: auth_state,
                    })

            })
        }
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
use reqwest::Method;

struct Request {
    inner: Arc<RequestRef>,
}

struct RequestRef {
    url: String,
    params: BTreeMap<String, String>,
    client: Client,
    ratelimit: Option<Ratelimit>,
    method: Method,
}

enum RequestState<T> {
    Uninitalized,
    WaitAuth(Box<dyn Future<Item=<AuthWaiter as Waiter>::Item, Error=<AuthWaiter as Waiter>::Error> + Send>),
    PollParse(Box<dyn Future<Item=T, Error=reqwest::Error> + Send>),
}

pub struct ApiRequest<T> {
    inner: Arc<RequestRef>,
    state: RequestState<T>
}

impl<T: DeserializeOwned + 'static + Send> ApiRequest<T> {

    pub fn new(url: String,
               params: BTreeMap<String, String>,
               client: Client,
               method: Method,
               ) -> ApiRequest<T>
    {
        ApiRequest {
            inner: Arc::new( RequestRef {
                url: url,
                params: params,
                client: client,
                method: method,
                ratelimit: None,
            }),
            state: RequestState::Uninitalized
        }
    }
}


use futures::Poll;
use serde::de::DeserializeOwned;
use futures::Async;
use futures::try_ready;


struct AuthWaiter {
    waiter: Client,
}


pub struct RatelimitWaiter {
    limit: Ratelimit,
    request: Request,
}

#[derive(Debug, Clone)]
pub struct Ratelimit {
    inner: Arc<Mutex<RatelimitRef>>    
}

#[derive(Debug, Clone)]
pub struct RatelimitRef {
    remaining: i32,
    inflight: i32,
    quota: i32,
    reset: Option<u32>,
}

use crate::sync::waiter::Waiter;
use crate::sync::barrier::{BarrierSync, Barrier};

impl Waiter for AuthWaiter {
    type Item = Self;
    type Error = Error;
    type ConditionError = ();

    fn blocked(&self) -> bool {
        let mut_client = self.waiter.inner.inner.lock().unwrap();
        mut_client.auth_state == AuthState::Unauth
    }

    fn condition_poller(&self) 
        -> Box<Future<Item=(), Error=Self::ConditionError> + Send> 
    {
        let bottom_client = self.waiter.get_bottom_client();
        let secret = self.waiter.inner.secret.as_ref().unwrap();
        let client = self.waiter.clone();

        let auth_future = 
            bottom_client
            .auth()
            .client_credentials(secret)
            .map(move |credentials| {
                println!("{:?}", credentials);
                let mut mut_client = client.inner.inner.lock().unwrap();
                mut_client.auth_state = AuthState::Auth;
                mut_client.token = Some(credentials.access_token.clone());
                ()
            })
            .map_err(|_| ());

        Box::new(auth_future)
    }

    fn into_future(self) -> Box<Future<Item=Self::Item, Error=Self::Error> + Send> {
        Box::new(futures::future::ok(self))
    }
}

impl Waiter for RatelimitWaiter {
    type Item = reqwest::r#async::Response;
    type Error = Error;
    type ConditionError = ();

    fn blocked(&self) -> bool {
        let limits = self.limit.inner.lock().unwrap();
        limits.remaining - limits.inflight <= 0
    }

    fn condition_poller(&self)
        -> Box<Future<Item=(), Error=Self::ConditionError> + Send> 
    {
        /*TODO: Really basic for now*/
        use futures_timer::Delay;
        use std::time::Duration;
        Box::new(
            Delay::new(Duration::from_secs(60))
                .map_err(|_| ())
        )
    }

    fn into_future(self) -> Box<Future<Item=Self::Item, Error=Self::Error> + Send> {
        let client = &self.request.inner.client;
        let reqwest = client.client();
        let method = &self.request.inner.method;
        let url = &self.request.inner.url;
        let params = &self.request.inner.params;

        let builder = reqwest.request(method.clone(), url);
        let builder = client.apply_standard_headers(builder);
        let r = builder.query(params);

        let limits = &self.limit.clone();

        /* TODO update limits */
        Box::new(r.send().map_err(|err| Error::from(err)))
    }

}

/* Todo: If the polled futures returns an error than all the waiters should
 * get that error
 */

impl<T: DeserializeOwned + 'static + Send> Future for ApiRequest<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match &mut self.state {
                RequestState::Uninitalized => {
                    let mut mut_client = self.inner.client.inner.inner.lock().unwrap();
                    let waiter = AuthWaiter {
                        waiter: self.inner.client.clone()
                    };

                    let f = mut_client.auth_barrier.wait_for(waiter);
                    self.state = RequestState::WaitAuth(f);
                },
                RequestState::WaitAuth(chan) => {
                    let _waiter = try_ready!(chan.poll());

                    let client = &self.inner.client;
                    let reqwest = client.client();

                    let builder = reqwest.request(self.inner.method.clone(), &self.inner.url);
                    let builder = client.apply_standard_headers(builder);
                    let r = builder.query(&self.inner.params);
                    
                    let f = r.send()
                            .map(|mut response| {
                                println!("{:?}", response);
                                response.json::<T>()
                            })
                            .and_then(|json| {
                                json
                            });

                    self.state = RequestState::PollParse(Box::new(f));
                    continue;
                }
                RequestState::PollParse(future) => {
                    let res = try_ready!(future.poll());
                    return Ok(Async::Ready(res));
                }
            }
        }
    }
}
