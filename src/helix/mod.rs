use futures::future::Future;
use std::sync::{Arc, Mutex};
use reqwest::r#async::Client as ReqwestClient;

use std::collections::HashSet;
use super::error::Error;
use std::marker::PhantomData;
use futures::future::Shared;
use futures::Poll;
use serde::de::DeserializeOwned;
use futures::Async;
use futures::try_ready;

use crate::error::ConditionError;


pub use super::types;

pub mod models;
pub mod namespaces;

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

#[derive(Clone, PartialEq)]
enum AuthState {
    Unauth,
    Auth,
}


struct MutClientRef {
    token: Option<String>,
    scopes: Vec<Scope>,
    previous: Option<Client>,
    auth_state: AuthState,
    auth_future: Option<Shared<Box<Future<Item=(), Error=ConditionError> + Send>>>
}

struct ClientRef {
    id: String,
    secret: Option<String>,
    client: ReqwestClient,
    auth_barrier: Barrier,
    ratelimit_default: Ratelimit,
    inner: Mutex<MutClientRef>,
}

impl Client {
    pub fn new(id: &str) -> Client {
        let client = ReqwestClient::new();
        Client::new_with_client(id, client)
    }

    pub fn default_ratelimit(&self) -> Ratelimit {
        self.inner.ratelimit_default.clone()
    }

    pub fn new_with_client(id: &str, client: ReqwestClient) -> Client {

        Client {
            inner: Arc::new(ClientRef {
                id: id.to_owned(),
                client: client,
                secret: None,
                auth_barrier: Barrier::new(),
                ratelimit_default: Ratelimit::new(30, "Ratelimit-Limit", "Ratelimit-Remaining", "Ratelimit-Reset"),
                inner: Mutex::new(
                    MutClientRef {
                        token: None,
                        scopes: Vec::new(),
                        previous: None,
                        auth_state: AuthState::Auth,
                        auth_future: None,
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
                auth_barrier: Barrier::new(),
                ratelimit_default: old_client.default_ratelimit(),
                inner: Mutex::new (
                    MutClientRef {
                        token: self.token,
                        scopes: Vec::new(),
                        previous: Some(old_client),
                        auth_state: auth_state,
                        auth_future: None,
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
    WaitAuth(WaiterState<AuthWaiter>),
    WaitLimit(WaiterState<RatelimitWaiter>),
    WaitRequest,
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
               ratelimit: Option<Ratelimit>,
               ) -> ApiRequest<T>
    {
        ApiRequest {
            inner: Arc::new( RequestRef {
                url: url,
                params: params,
                client: client,
                method: method,
                ratelimit: ratelimit,
            }),
            state: RequestState::Uninitalized
        }
    }
}


pub struct RatelimitWaiter {
    limit: Ratelimit,
}

#[derive(Clone)]
pub struct Ratelimit {
    inner: Arc<Mutex<RatelimitRef>>,
    barrier: Barrier,
}

impl Ratelimit {
    pub fn new(limit: i32,
            header_limit: &str,
            header_remaining: &str,
            header_reset: &str)
        -> Ratelimit 
    {
        Ratelimit {
            inner: Arc::new(
                       Mutex::new(
                           RatelimitRef {
                                limit: limit,
                                remaining: limit,
                                inflight: 0,
                                reset: None,
                                header_limit: header_limit.to_owned(),
                                header_remaining: header_remaining.to_owned(),
                                header_reset: header_reset.to_owned(),
                           }
                        )
                    ),
            barrier: Barrier::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RatelimitRef {
    limit: i32,
    remaining: i32,
    inflight: i32,
    reset: Option<u32>,
    header_limit: String,
    header_remaining: String,
    header_reset: String,
}

use futures::future::SharedError;
use crate::sync::barrier::Barrier;
use crate::sync::waiter::Waiter;

struct WaiterState<W: Waiter> {
    polling: bool,
    shared_future: Option<(Shared<Box< Future<Item=(), Error=ConditionError> + Send>>)>,
    waiter: W,
    barrier: Barrier,
}

impl<W: Waiter> WaiterState<W> {
    fn new(waiter: W, barrier: &Barrier) -> WaiterState<W> {
        WaiterState {
            polling: false,
            shared_future: None,
            waiter: waiter,
            barrier: barrier.clone(),
        }
    }
}

impl<W: Waiter> Future for WaiterState<W> {
    type Item = <W as Waiter>::Item;
    type Error = <W as Waiter>::Error; 

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let blocked = self.waiter.blocked();
            if blocked && !self.polling {
                let fut = self.barrier.condition(&self.waiter);
                self.shared_future = Some(fut);
                self.polling = true;
            } else if blocked || self.polling {
                let f = self.shared_future.as_mut().unwrap();
                try_ready!(f.poll());
                self.polling = false;
            } else {
                return Ok(Async::Ready(<W as Waiter>::Item::default()));
            }
        }
    }
}


struct AuthWaiter {
    waiter: Client,
}

impl Waiter for AuthWaiter {
    type Item = ();
    type Error = ConditionError;

    fn blocked(&self) -> bool {
        let mut_client = self.waiter.inner.inner.lock().unwrap();
        mut_client.auth_state == AuthState::Unauth
    }

    fn condition(&self) ->
        Shared<Box<Future<Item=(), Error=ConditionError> + Send>> {
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
            .map_err(|_| ConditionError{});

        Future::shared(Box::new(auth_future))
    }
}

impl Waiter for RatelimitWaiter {
    type Item = ();
    type Error = ConditionError;

    fn blocked(&self) -> bool {
        let limits = self.limit.inner.lock().unwrap();
        println!("{}, {}, {}", limits.limit, limits.remaining, limits.inflight);
        limits.remaining - limits.inflight <= 0
    }

    fn condition(&self)
        -> Shared<Box<Future<Item=(), Error=ConditionError> + Send>> 
    {
        /*TODO: Really basic for now*/
        use futures_timer::Delay;
        use std::time::Duration;
        Future::shared(Box::new(
            Delay::new(Duration::from_secs(60)).map_err(|_| ConditionError{})
        ))
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
                    let mut_client = self.inner.client.inner.inner.lock().unwrap();

                    let waiter = AuthWaiter {
                        waiter: self.inner.client.clone(),
                    };

                    let f = WaiterState::new(waiter,
                                &self.inner.client.inner.auth_barrier);
                    self.state = RequestState::WaitAuth(f);
                },
                RequestState::WaitAuth(auth) => {
                    let _waiter = try_ready!(auth.poll());
                    match self.inner.ratelimit {
                        Some(ref limit) => {
                            let barrier = limit.barrier.clone();
                            let waiter = RatelimitWaiter {
                                limit: limit.clone(),
                            };
                            let f = WaiterState::new(waiter,
                                        &barrier);
                            self.state = RequestState::WaitLimit(f);
                        },
                        None => {
                            self.state = RequestState::WaitRequest;
                        }
                    }
                },
                RequestState::WaitLimit(limit) => {
                    let _waiter = try_ready!(limit.poll());
                    self.state = RequestState::WaitRequest;
                }, 
                RequestState::WaitRequest => {
                    let client = &self.inner.client;
                    let reqwest = client.client();

                    if let Some(limits) = &self.inner.ratelimit {
                        let mut mut_limits = limits.inner.lock().unwrap();
                        mut_limits.inflight = mut_limits.inflight + 1;
                    }

                    let builder = reqwest.request(self.inner.method.clone(), &self.inner.url);
                    let builder = client.apply_standard_headers(builder);
                    let r = builder.query(&self.inner.params);
                    /*TODO add 1 to inflight*/

                    let ratelimit_err = self.inner.ratelimit.clone();
                    let ratelimit_ok = self.inner.ratelimit.clone();
                    
                    let f = r.send()
                            .map_err(|err| {

                                if let Some(limits) = ratelimit_err {
                                    let mut mut_limits = limits.inner.lock().unwrap();
                                    mut_limits.inflight = mut_limits.inflight - 1;
                                }

                                err
                            })
                            .map(|mut response| {
                                println!("{:?}", response);
                                if let Some(limits) = ratelimit_ok {
                                    let mut mut_limits = limits.inner.lock().unwrap();
                                    mut_limits.inflight = mut_limits.inflight - 1;

                                    let maybe_limit =
                                        response.headers()
                                        .get(&mut_limits.header_limit)
                                        .and_then(|x| x.to_str().ok())
                                        .and_then(|x| x.parse::<i32>().ok());

                                    if let Some(limit) = maybe_limit {
                                        mut_limits.limit = limit;
                                    }

                                    let maybe_remaining = 
                                        response.headers()
                                        .get(&mut_limits.header_limit)
                                        .and_then(|x| x.to_str().ok())
                                        .and_then(|x| x.parse::<i32>().ok());

                                    if let Some(limit) = maybe_remaining {
                                        mut_limits.remaining = limit;
                                    }

                                    let maybe_reset =
                                        response.headers()
                                        .get(&mut_limits.header_limit)
                                        .and_then(|x| x.to_str().ok())
                                        .and_then(|x| x.parse::<u32>().ok());

                                    if let Some(reset) = maybe_reset {
                                        mut_limits.reset = Some(reset);
                                    }
                                }

                                response.json::<T>()
                            })
                            .and_then(|json| {
                                json
                            });

                    self.state = RequestState::PollParse(Box::new(f));
                },
                RequestState::PollParse(future) => {
                    let res = try_ready!(future.poll());
                    return Ok(Async::Ready(res));
                }
            }
        }
    }
}
