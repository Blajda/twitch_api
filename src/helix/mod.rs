use futures::future::Future;
use std::sync::{Arc, Mutex};
use reqwest::r#async::Client as ReqwestClient;

use std::collections::{HashSet, HashMap};
use super::error::Error;
use futures::future::Shared;
use futures::Poll;
use serde::de::DeserializeOwned;
use futures::Async;
use futures::try_ready;
use std::iter::FromIterator;

use crate::error::ConditionError;


pub use super::types;

pub mod models;
pub mod namespaces;

const API_DOMAIN: &'static str = "api.twitch.tv";

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum RatelimitKey {
    Default,
}
type RatelimitMap = HashMap<RatelimitKey, Ratelimit>;

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
    inner: Arc<ClientType>,
}

enum ClientType {
    Unauth(UnauthClient),
    Auth(AuthClient),
}

/*TODO: Try to remove this boilerplate too*/
impl ClientTrait for Client {

    fn id<'a>(&'a self) -> &'a str {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.id(),
            Auth(inner) => inner.id(),
        }
    }

    fn domain<'a>(&'a self) -> &'a str {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.domain(),
            Auth(inner) => inner.domain(),
        }
    }

    fn ratelimit<'a>(&self, key: RatelimitKey) -> Option<&'a Ratelimit> {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.ratelimit(key),
            Auth(inner) => inner.ratelimit(key),
        }
    }

    fn authenticated(&self) -> bool {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.authenticated(),
            Auth(inner) => inner.authenticated(),
        }
    }

    fn scopes(&self) -> Vec<Scope> {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.scopes(),
            Auth(inner) => inner.scopes(),
        }
    }
}

pub struct UnauthClient {
    id: String,
    reqwest: ReqwestClient,
    domain: String,
    ratelimits: RatelimitMap,
}

impl Client {

    pub fn authenticate(self, secret: &str) -> AuthClientBuilder {
        AuthClientBuilder::new(self, secret)
    }

    pub fn deauthenticate(self) -> Client {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(_inner) => self,
            Auth(inner) => inner.previous.clone(),
        }
    }
}


pub trait ClientTrait {

    fn id<'a>(&'a self) -> &'a str;
    fn domain<'a>(&'a self) -> &'a str;
    fn ratelimit<'a>(&self, key: RatelimitKey) -> Option<&'a Ratelimit>;

    fn authenticated(&self) -> bool;
    fn scopes(&self) -> Vec<Scope>;
}

impl ClientTrait for UnauthClient {
    fn id<'a>(&'a self) -> &'a str {
        &self.id
    }

    fn domain<'a>(&'a self) -> &'a str {
        &self.domain
    }

    fn ratelimit<'a>(&self, key: RatelimitKey) -> Option<&'a Ratelimit> {
        None
    }

    fn authenticated(&self) -> bool {
        false
    }

    fn scopes(&self) -> Vec<Scope> {
        Vec::with_capacity(0)
    }
}

pub struct AuthClient {
    secret: String,
    auth_state: Mutex<AuthStateRef>,
    auth_barrier: Barrier,
    previous: Client,
}

/*TODO I'd be nice to remove this boiler plate */
impl ClientTrait for AuthClient {
    fn id<'a>(&'a self) -> &'a str {
        match self.previous.inner.as_ref() {
            ClientType::Auth(auth) => auth.id(),
            ClientType::Unauth(unauth) => unauth.id(),
        }
    }

    fn domain<'a>(&'a self) -> &'a str {
        match self.previous.inner.as_ref() {
            ClientType::Auth(auth) => auth.domain(),
            ClientType::Unauth(unauth) => unauth.domain(),
        }
    }

    fn ratelimit<'a>(&self, key: RatelimitKey) -> Option<&'a Ratelimit> {
        match self.previous.inner.as_ref() {
            ClientType::Auth(auth) => auth.ratelimit(key),
            ClientType::Unauth(unauth) => unauth.ratelimit(key),
        }
    }

    fn authenticated(&self) -> bool {
        let auth = self.auth_state.lock().expect("Auth Lock is poisoned");
        auth.state == AuthState::Auth
    }

    fn scopes(&self) -> Vec<Scope> {
        let auth = self.auth_state.lock().expect("Auth Lock is poisoned");
        Vec::with_capacity(0)
    }
}

#[derive(Clone, PartialEq)]
enum AuthState {
    Unauth,
    Auth,
}

struct AuthStateRef {
    token: Option<String>,
    scopes: Vec<Scope>,
    state: AuthState,
}

struct ClientRef {
    id: String,
    secret: Option<String>,
    reqwest: ReqwestClient,
    domain: &'static str,
    ratelimits: RatelimitMap,
    auth_state: Mutex<AuthStateRef>,
    auth_barrier: Barrier,
    previous: Option<Client>,
}

impl Client {
    pub fn new(id: &str) -> Client {
        let client = ReqwestClient::new();
        Client::new_with_client(id, client)
    }

    fn default_ratelimits() -> RatelimitMap {
        let mut limits = RatelimitMap::new();
        limits.insert(RatelimitKey::Default, Ratelimit::new(30, "Ratelimit-Limit", "Ratelimit-Remaining", "Ratelimit-Reset"));

        limits
    }

    pub fn new_with_client(id: &str, reqwest: ReqwestClient) -> Client {

        Client {
            inner: Arc::new(
                ClientType::Unauth(UnauthClient {
                    id: id.to_owned(),
                    reqwest: reqwest,
                    domain: API_DOMAIN.to_owned(),
                    ratelimits: Self::default_ratelimits(),
            }))
        }
    }

    fn secret<'a>(&'a self) -> Option<&'a str> {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(_) => None,
            Auth(inner) => Some(&inner.secret),
        }
    }

    fn reqwest(&self) -> ReqwestClient {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.reqwest.clone(),
            Auth(inner) => inner.previous.reqwest(),
        }
    }

    /* The 'bottom' client must always be a client that is not authorized.
     * This which allows for calls to Auth endpoints using the same control flow
     * as other requests.
     *
     * Clients created with 'new' are bottom clients and calls
     * to authenticate stack a authed client on top
     */
    fn get_bottom_client(&self) -> Client {
        match self.inner.as_ref() {
            ClientType::Auth(inner) => inner.previous.get_bottom_client(),
            ClientType::Unauth(_) => self.clone(),
        }
    }

    fn apply_standard_headers(&self, request: RequestBuilder) 
       -> RequestBuilder 
    {
        let token = match self.inner.as_ref() {
            ClientType::Auth(inner) => {
                let auth = inner.auth_state.lock().expect("Authlock is poisoned");
                auth.token.as_ref().map(|s| s.to_owned())
            }
            ClientType::Unauth(_) => None,
        };

        let client_header = header::HeaderValue::from_str(self.id()).unwrap();

        let request =
            if let Some(token) = token {
                let value = "Bearer ".to_owned() + &token;
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
            inner: Arc::new(ClientType::Auth(
                AuthClient {
                secret: self.secret,
                auth_barrier: Barrier::new(),
                auth_state: Mutex::new (
                    AuthStateRef {
                        token: self.token,
                        scopes: Vec::new(),
                        state: auth_state,
                    }),
                previous: old_client,
            }))
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
    ratelimit: Option<RatelimitKey>,
    method: Method,
}

enum RequestState<T> {
    Uninitalized,
    WaitAuth(WaiterState<AuthWaiter>),
    SetupRatelimit,
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
               params: BTreeMap<&str, &str>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>,
               ) -> ApiRequest<T>
    {
        let mut owned_params = BTreeMap::new();
        for (key, value) in params {
            owned_params.insert(key.to_owned(), value.to_owned());
        }

        ApiRequest {
            inner: Arc::new( RequestRef {
                url: url,
                params: owned_params,
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
        match self.waiter.inner.as_ref() {
            ClientType::Unauth(_) => false,
            ClientType::Auth(inner) => {
                let auth = inner.auth_state.lock()
                    .expect("unable to lock auth state");
                auth.state == AuthState::Unauth
            }
        }
    }

    fn condition(&self) ->
        Shared<Box<Future<Item=(), Error=ConditionError> + Send>> {
        /* If a secret is not provided than just immediately return */
        let secret = self.waiter.secret().unwrap();
        let bottom_client = self.waiter.get_bottom_client();
        let client = self.waiter.clone();

        let auth_future = 
            bottom_client
            .auth()
            .client_credentials(secret)
            .map(move |credentials| {
                println!("{:?}", credentials);
                if let ClientType::Auth(inner) = client.inner.as_ref() {
                    let mut auth = inner.auth_state.lock().unwrap();
                    auth.state = AuthState::Auth;
                    auth.token = Some(credentials.access_token.clone());
                }
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
                    match self.inner.client.inner.as_ref() {
                        ClientType::Auth(inner) => {
                            let waiter = AuthWaiter {
                                waiter: self.inner.client.clone(),
                            };

                            let f = WaiterState::new(waiter,
                                        &inner.auth_barrier);
                            self.state = RequestState::WaitAuth(f);
                        },
                        ClientType::Unauth(_) => {
                            self.state = RequestState::SetupRatelimit;
                        }
                    }
                },
                RequestState::WaitAuth(auth) => {
                    let _waiter = try_ready!(auth.poll());
                    self.state = RequestState::SetupRatelimit;
                },
                RequestState::SetupRatelimit => {
                    let limits = 
                        self.inner.ratelimit.as_ref().and_then(|key| {
                            self.inner.client.ratelimit(key.clone())
                        });
                    match limits {
                        Some(ratelimit) => {
                            let barrier = ratelimit.barrier.clone();
                            let waiter = RatelimitWaiter {
                                limit: ratelimit.clone(),
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
                    let reqwest = client.reqwest();

                    let limits = 
                        self.inner.ratelimit.as_ref().and_then(|key| {
                            client.ratelimit(key.clone())
                        });

                    if let Some(limits) = limits {
                        let mut mut_limits = limits.inner.lock().unwrap();
                        mut_limits.inflight = mut_limits.inflight + 1;
                    }

                    let builder = reqwest.request(self.inner.method.clone(), &self.inner.url);
                    let builder = client.apply_standard_headers(builder);
                    let r = builder.query(&self.inner.params);

                    let limits_err = limits.clone();
                    let limits_ok = limits.clone();
                    
                    let f = r.send()
                            .map_err(move |err| {

                                if let Some(limits) = limits_err {
                                    let mut mut_limits = limits.inner.lock().unwrap();
                                    mut_limits.inflight = mut_limits.inflight - 1;
                                }

                                err
                            })
                            .map(move |mut response| {
                                println!("{:?}", response);
                                if let Some(limits) = limits_ok {
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
                                        .get(&mut_limits.header_remaining)
                                        .and_then(|x| x.to_str().ok())
                                        .and_then(|x| x.parse::<i32>().ok());

                                    if let Some(limit) = maybe_remaining {
                                        mut_limits.remaining = limit;
                                    }

                                    let maybe_reset =
                                        response.headers()
                                        .get(&mut_limits.header_reset)
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
