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

use crate::error::ConditionError;

pub use super::types;

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum RatelimitKey {
    Default,
}
type RatelimitMap = HashMap<RatelimitKey, Ratelimit>;

const API_DOMAIN: &'static str = "api.twitch.tv";
const AUTH_DOMAIN: &'static str = "id.twitch.tv";
const KRAKEN_ACCEPT: &'static str = "application/vnd.twitchtv.v5+json";

pub trait PaginationTrait {
    fn cursor<'a>(&'a self) -> Option<&'a str>;
}


#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientType>,
}

/*TODO*/
#[derive(PartialEq, Hash, Eq, Clone)]
pub enum Scope {
    UserReadEmail,
}

#[derive(Clone)]
pub enum Version {
    Helix,
    Kraken,
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

enum ClientType {
    Unauth(UnauthClient),
    Auth(AuthClient),
}

pub struct UnauthClient {
    id: String,
    reqwest: ReqwestClient,
    domain: String,
    auth_domain: String,
    ratelimits: RatelimitMap,
    version: Version,
}

pub struct AuthClient {
    secret: String,
    auth_state: Mutex<AuthStateRef>,
    auth_barrier: Barrier,
    previous: Client,
}

pub trait ClientTrait {

    fn id<'a>(&'a self) -> &'a str;
    fn domain<'a>(&'a self) -> &'a str;
    fn auth_domain<'a>(&'a self) -> &'a str;
    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a Ratelimit>;

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

    fn auth_domain<'a>(&'a self) -> &'a str {
        &self.auth_domain
    }

    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a Ratelimit> {
        self.ratelimits.get(&key)
    }

    fn authenticated(&self) -> bool {
        false
    }

    fn scopes(&self) -> Vec<Scope> {
        Vec::with_capacity(0)
    }
}

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

    fn auth_domain<'a>(&'a self) -> &'a str {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.auth_domain(),
            Auth(inner) => inner.auth_domain(),
        }
    }

    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a Ratelimit> {
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

    fn auth_domain<'a>(&'a self) -> &'a str {
        match self.previous.inner.as_ref() {
            ClientType::Auth(auth) => auth.auth_domain(),
            ClientType::Unauth(unauth) => unauth.auth_domain(),
        }
    }

    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a Ratelimit> {
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

impl Client {
    pub fn new(id: &str, version: Version) -> Client {
        let client = ReqwestClient::new();
        Client::new_with_client(id, client, version)
    }

    fn default_ratelimits() -> RatelimitMap {
        let mut limits = RatelimitMap::new();
        limits.insert(RatelimitKey::Default, Ratelimit::new(30, "Ratelimit-Limit", "Ratelimit-Remaining", "Ratelimit-Reset"));

        limits
    }

    pub fn new_with_client(id: &str, reqwest: ReqwestClient, version: Version) -> Client {

        Client {
            inner: Arc::new(
                ClientType::Unauth(UnauthClient {
                    id: id.to_owned(),
                    reqwest: reqwest,
                    domain: API_DOMAIN.to_owned(),
                    auth_domain: AUTH_DOMAIN.to_owned(),
                    ratelimits: Self::default_ratelimits(),
                    version: version,
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

    fn version(&self) -> Version {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.version.clone(),
            Auth(inner) => inner.previous.version(),
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
     * to authenticate stack an authed client on top
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
        match self.version() {
            Version::Helix => {

                let client_header = header::HeaderValue::from_str(self.id()).unwrap();

                let request =
                    if let Some(token) = token {
                        let value = "Bearer ".to_owned() + &token;
                        let token_header = header::HeaderValue::from_str(&value).unwrap();
                        request.header("Authorization", token_header)
                    } else { request };

                request.header("Client-ID", client_header)
            },
            Version::Kraken => {
                let client_header = header::HeaderValue::from_str(self.id()).unwrap();
                let accept_header = header::HeaderValue::from_str(KRAKEN_ACCEPT).unwrap();

                let request = request.header("Client-ID", client_header);
                let request = request.header("Accept", accept_header);
                let request = if let Some(token) = token {
                    let value = "OAuth ".to_owned() + &token;
                    let token_header = header::HeaderValue::from_str(&value).unwrap();
                    request.header("Authorization", token_header)
                } else {request};

                request
            }
        }
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

struct RequestRef {
    url: String,
    params: BTreeMap<String, String>,
    client: Client,
    ratelimit: Option<RatelimitKey>,
    method: Method,
}

impl RequestRef {
    pub fn new(url: String,
               params: BTreeMap<&str, &str>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>,
               ) -> RequestRef 
    {
        let mut owned_params = BTreeMap::new();
        for (key, value) in params {
            owned_params.insert(key.to_owned(), value.to_owned());
        }

        RequestRef {
            url: url,
            params: owned_params,
            client: client,
            method: method,
            ratelimit: ratelimit,
        }
    }
}

enum RequestState<T> {
    SetupRequest,
    SetupBarriers,
    WaitAuth(WaiterState<AuthWaiter>),
    SetupRatelimit,
    WaitLimit(WaiterState<RatelimitWaiter>),
    WaitRequest,
    PollParse(Box<dyn Future<Item=T, Error=reqwest::Error> + Send>),
}

pub struct ApiRequest<T> {
    inner: Arc<RequestRef>,
    state: RequestState<T>,
    attempt: u32,
    max_attempts: u32,
    pagination: Option<String>,
}

enum IterableApiRequestState<T> {
    Start,
    PollInner(ApiRequest<T>),
    Finished,
}

pub struct IterableApiRequest<T> {
    inner: Arc<RequestRef>,
    state: IterableApiRequestState<T>, 
}

impl<T: DeserializeOwned + PaginationTrait + 'static + Send> ApiRequest<T> {

    pub fn new(url: String,
               params: BTreeMap<&str, &str>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>,
               ) -> ApiRequest<T>
    {
        ApiRequest {
            inner: Arc::new(RequestRef::new(url, params, client, method, ratelimit)),
            state: RequestState::SetupRequest,
            attempt: 0,
            max_attempts: 1,
            pagination: None,
        }
    }
}

impl<T: DeserializeOwned + PaginationTrait + 'static + Send> IterableApiRequest<T> {
    
    pub fn new(url: String,
               params: BTreeMap<&str, &str>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>
               ) -> IterableApiRequest<T>
    {
        let request_ref =
            Arc::new(RequestRef::new(url, params, client, method, ratelimit));

        IterableApiRequest {
            inner: request_ref,
            state: IterableApiRequestState::Start,
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


impl RatelimitRef {
    pub fn update_from_headers(&mut self, headers: &reqwest::header::HeaderMap) {
        let maybe_limit =
            headers
            .get(&self.header_limit)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<i32>().ok());

        if let Some(limit) = maybe_limit {
            self.limit = limit;
        }

        let maybe_remaining = 
            headers
            .get(&self.header_remaining)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<i32>().ok());

        if let Some(limit) = maybe_remaining {
            self.remaining = limit;
        }

        let maybe_reset =
            headers
            .get(&self.header_reset)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<u32>().ok());

        if let Some(reset) = maybe_reset {
            self.reset = Some(reset);
        }
    }
}

use futures::future::SharedError;
use crate::sync::barrier::Barrier;
use crate::sync::waiter::Waiter;

struct WaiterState<W: Waiter> {
    polling: bool,
    shared_future: Option<(Shared<Box<Future<Item=(), Error=ConditionError> + Send>>)>,
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
        /* If a secret is not provided then just immediately return */
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
        let limits = self.limit.clone();
        Future::shared(Box::new(
            Delay::new(Duration::from_secs(60))
            .map(move |_res| {
                let mut limits = limits.inner.lock().unwrap();
                limits.remaining = limits.limit;
                ()
            })
            .map_err(|_| ConditionError{})
        ))
    }
}

/* Todo: If the polled futures returns an error than all the waiters should
 * get that error
 */

/* Macro ripped directly from try_ready and simplies retries if any error occurs
 * and there are remaning retry attempt
 */
#[macro_export]
macro_rules! retry_ready {
    ($s:expr, $e:expr) => (match $e {
        Ok(futures::prelude::Async::Ready(t)) => t,
        Ok(futures::prelude::Async::NotReady) => return Ok(futures::prelude::Async::NotReady),
        Err(e) => {
            if $s.attempt < $s.max_attempts {
                $s.attempt += 1;
                $s.state = RequestState::SetupBarriers;
                continue;
            } else {
                return Err(e.into());
            }
        }
    })
}

use futures::Stream;

impl<T: DeserializeOwned + PaginationTrait + 'static + Send> Stream for IterableApiRequest<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match &mut self.state {
                IterableApiRequestState::Start => {
                    self.state = 
                        IterableApiRequestState::PollInner(
                            ApiRequest {
                                inner: self.inner.clone(),
                                state: RequestState::SetupRequest,
                                attempt: 0,
                                max_attempts: 1,
                                pagination: None
                            });
                },
                IterableApiRequestState::PollInner(request) => {
                    let f = request as &mut Future<Item=Self::Item, Error=Self::Error>;
                    match f.poll() {
                        Err(err) => {
                            self.state = IterableApiRequestState::Finished;
                            return Err(err);
                        },
                        Ok(state) => {
                            match state {
                                Async::NotReady => return Ok(Async::NotReady),
                                Async::Ready(res) => {
                                    let cursor = res.cursor();
                                    match cursor {
                                        Some(cursor) => {
                                            self.state = IterableApiRequestState::PollInner(
                                                ApiRequest {
                                                    inner: self.inner.clone(),
                                                    state: RequestState::SetupRequest,
                                                    attempt: 0,
                                                    max_attempts: 1,
                                                    pagination: Some(cursor.to_owned()),
                                                });
                                        },
                                        None => {
                                            self.state = IterableApiRequestState::Finished;
                                        }
                                    }
                                    return Ok(Async::Ready(Some(res)));
                                }
                            }
                        }
                    }
                },
                IterableApiRequestState::Finished => {
                    return Ok(Async::Ready(None));
                }
            }
        }
    }
}

impl<T: DeserializeOwned + 'static + Send> Future for ApiRequest<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match &mut self.state {
                RequestState::SetupRequest => {
                    self.attempt = 0;
                    self.state = RequestState::SetupBarriers;
                }
                RequestState::SetupBarriers => {
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
                    let _waiter = retry_ready!(self, auth.poll());
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
                    let _waiter = retry_ready!(self, limit.poll());
                    self.state = RequestState::WaitRequest;
                }, 
                RequestState::WaitRequest => {
                    let client = self.inner.client.clone();
                    let c_ref = &client;
                    let reqwest = client.reqwest();

                    let limits = 
                        self.inner.ratelimit.as_ref().and_then(|key| {
                            c_ref.ratelimit(key.clone())
                        });

                    if let Some(limits) = limits {
                        let mut mut_limits = limits.inner.lock().unwrap();
                        mut_limits.inflight = mut_limits.inflight + 1;
                    }

                    let mut builder = reqwest.request(self.inner.method.clone(), &self.inner.url);
                    builder = client.apply_standard_headers(builder);
                    builder = builder.query(&self.inner.params);
                    builder = 
                        if let Some(cursor) = &self.pagination {
                            builder.query(&[("after", cursor)])
                        } else {
                            builder
                        };
                     

                    let key_err = self.inner.ratelimit.clone();
                    let key_ok = self.inner.ratelimit.clone();
                    let client_err = client.clone();
                    let client_ok = client.clone();
                    
                    let f = 
                        builder.send()
                            .map_err(move |err| {
                                if let Some(key) = key_err {
                                    if let Some(limits) = client_err.ratelimit(key) {
                                        let mut mut_limits = limits.inner.lock().unwrap();
                                        mut_limits.inflight = mut_limits.inflight - 1;
                                    }
                                }

                                err
                            })
                            .map(move |mut response| {
                                println!("{:?}", response);
                                if let Some(key) = key_ok {
                                    if let Some(limits) = client_ok.ratelimit(key) {
                                        let mut mut_limits = limits.inner.lock().unwrap();
                                        mut_limits.inflight = mut_limits.inflight - 1;
                                        mut_limits.update_from_headers(response.headers());
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
                    let res = retry_ready!(self, future.poll());
                    return Ok(Async::Ready(res));
                },
            }
        }
    }
}
