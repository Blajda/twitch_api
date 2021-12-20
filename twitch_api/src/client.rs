use crate::models::Message;
use std::convert::TryFrom;
use std::sync::{Arc, Mutex};

use reqwest::r#async::Client as ReqwestClient;
use reqwest::r#async::{Response};
use reqwest::r#async::{RequestBuilder};
use reqwest::header;


use std::collections::{HashSet, HashMap};
use super::error::Error;

use futuresv01::Stream;
use futuresv01::future::Future as Futurev01;
use futuresv01::future::Shared;
use futuresv01::Poll;
use futuresv01::Async;
use futuresv01::try_ready;
use futuresv01::future::Either;

use std::future::Future;
use std::future::IntoFuture;

use futures::compat::Compat01As03;

use serde::de::DeserializeOwned;

use crate::error::ConditionError;


#[derive(PartialEq, Eq, Hash, Clone)]
pub enum RatelimitKey {
    Default,
}

pub struct RatelimitMap {
    pub inner: HashMap<RatelimitKey, Ratelimit>
}

const API_BASE_URI: &'static str = "https://api.twitch.tv";
const AUTH_BASE_URI: &'static str = "https://id.twitch.tv";

pub trait PaginationTrait {
    fn cursor<'a>(&'a self) -> Option<&'a str>;
}

pub type ParamList<'a> = BTreeMap<&'a str, &'a dyn ToString>;

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientType>,
}

#[derive(Debug)]
pub struct ScopeParseError {}
use std::fmt;
impl fmt::Display for ScopeParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Scope Parse Error")
    }
}

/*TODO*/
#[derive(PartialEq, Hash, Eq, Clone, Debug)]
pub enum Scope {
    Helix(HelixScope),
}

impl TryFrom<&str> for Scope {
    type Error = ScopeParseError;
    fn try_from(s: &str) -> Result<Scope, Self::Error> {
        if let Ok(scope) = HelixScope::try_from(s) {
            return Ok(Scope::Helix(scope));
        }
        Err(ScopeParseError {})
    }
}
use serde::{Deserialize, Deserializer}; 
impl<'de> Deserialize<'de> for Scope {

    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de> 
    {
        let id = String::deserialize(deserializer)?;
        Scope::try_from(&id[0..]).map_err(serde::de::Error::custom)
    }
}

#[derive(PartialEq, Hash, Eq, Clone, Debug)]
pub enum HelixScope { 
    AnalyticsReadExtensions,
    AnalyticsReadGames,
    BitsRead,
    ChannelReadSubscriptions,
    ClipsEdit,
    UserEdit,
    UserEditBroadcast,
    UserReadBroadcast,
    UserReadEmail,
}

impl HelixScope {
    pub fn to_str(&self) -> &'static str {
        use self::HelixScope::*;
        match self {
            AnalyticsReadExtensions => "analytics:read:extensions",
            AnalyticsReadGames => "analytics:read:games",
            BitsRead => "bits:read",
            ChannelReadSubscriptions => "channel:read:subscriptions",
            ClipsEdit => "clips:edit",
            UserEdit => "user:edit",
            UserEditBroadcast => "user:edit:broadcast",
            UserReadBroadcast => "user:read:broadcast",
            UserReadEmail => "user:read:email",
        }
    }
}

impl TryFrom<&str> for HelixScope {
    type Error = ScopeParseError;
    fn try_from(s: &str) -> Result<HelixScope, Self::Error> {
        use self::HelixScope::*;
        Ok( match s {
            "analytics:read:extensions" => AnalyticsReadExtensions,
            "analytics:read:games" => AnalyticsReadGames,
            "bits:read" => BitsRead,
            "channel:read:subscriptions" => ChannelReadSubscriptions,
            "clips:edit" => ClipsEdit,
            "user:edit" => UserEdit,
            "user:edit:broadcast" => UserEditBroadcast,
            "user:read:broadcast" => UserReadBroadcast,
            "user:read:email" => UserReadEmail,
            _ => return Err(ScopeParseError{})
        })
    }
}

#[derive(Clone)]
pub enum Version {
    Helix,
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


pub struct ClientConfig {
    pub reqwest: ReqwestClient,
    pub api_base_uri:  String,
    pub auth_base_uri: String,
    pub ratelimits: RatelimitMap,
    pub max_retrys: u32,
}

impl Default for RatelimitMap {

    fn default() -> Self {
        let mut limits = HashMap::new();
        limits.insert(RatelimitKey::Default, Ratelimit::new(30, "ratelimit-limit", "ratelimit-remaining", "ratelimit-reset"));
        RatelimitMap {
            inner: limits
        }
    }
}

impl RatelimitMap {
    pub fn empty() -> RatelimitMap {
        RatelimitMap {
            inner: HashMap::new()
        }
    }
}

impl Default for ClientConfig {

    fn default() -> Self {
        let reqwest = ReqwestClient::new();
        let ratelimits = RatelimitMap::default();

        ClientConfig {
            reqwest,
            api_base_uri: API_BASE_URI.to_owned(),
            auth_base_uri: AUTH_BASE_URI.to_owned(),
            ratelimits,
            max_retrys: 1,
        }
    }
}

pub struct UnauthClient {
    id: String,
    config: ClientConfig,
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
    fn config<'a>(&'a self) -> &'a ClientConfig;
    fn api_base_uri<'a>(&'a self) -> &'a str;
    fn auth_base_uri<'a>(&'a self) -> &'a str;
    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a Ratelimit>;

    fn authenticated(&self) -> bool;
    fn scopes(&self) -> Vec<Scope>;
}

impl ClientTrait for UnauthClient {
    fn id<'a>(&'a self) -> &'a str {
        &self.id
    }

    fn api_base_uri<'a>(&'a self) -> &'a str {
        &self.config.api_base_uri
    }

    fn auth_base_uri<'a>(&'a self) -> &'a str {
        &self.config.auth_base_uri
    }

    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a Ratelimit> {
        self.config.ratelimits.inner.get(&key)
    }

    fn authenticated(&self) -> bool {
        false
    }

    fn config<'a>(&'a self) -> &'a ClientConfig {
        &self.config
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

    fn api_base_uri<'a>(&'a self) -> &'a str {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.api_base_uri(),
            Auth(inner) => inner.api_base_uri(),
        }
    }

    fn auth_base_uri<'a>(&'a self) -> &'a str {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.auth_base_uri(),
            Auth(inner) => inner.auth_base_uri(),
        }
    }

    fn config<'a>(&'a self) -> &'a ClientConfig {
        use self::ClientType::*;
        match self.inner.as_ref() {
            Unauth(inner) => inner.config(),
            Auth(inner) => inner.config(),
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

    fn api_base_uri<'a>(&'a self) -> &'a str {
        match self.previous.inner.as_ref() {
            ClientType::Auth(auth) => auth.api_base_uri(),
            ClientType::Unauth(unauth) => unauth.api_base_uri(),
        }
    }

    fn auth_base_uri<'a>(&'a self) -> &'a str {
        match self.previous.inner.as_ref() {
            ClientType::Auth(auth) => auth.auth_base_uri(),
            ClientType::Unauth(unauth) => unauth.auth_base_uri(),
        }
    }

    fn config<'a>(&'a self) -> &'a ClientConfig {
        match self.previous.inner.as_ref() {
            ClientType::Auth(auth) => auth.config(),
            ClientType::Unauth(unauth) => unauth.config(),
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
        auth.scopes.clone()
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
    pub fn new(id: &str, config: ClientConfig, version: Version) -> Client {
        Client {
            inner: Arc::new(
                ClientType::Unauth(UnauthClient {
                    id: id.to_owned(),
                    config: config,
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
            Unauth(inner) => inner.config.reqwest.clone(),
            Auth(inner) => inner.previous.reqwest(),
        }
    }

    fn send(&self, builder: RequestBuilder) -> Box<dyn Futurev01<Item=Response, Error=reqwest::Error> + Send> {
        Box::new(builder.send())
    }

    /* The 'bottom' client must always be a client that is not authorized.
     * This allows for calls to Auth endpoints using the same control flow
     * as other requests.
     *
     * Clients created with 'new' are bottom clients. Calls to
     * to 'authenticate' stack an authed client on top
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
        }
    }
}



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
               params: BTreeMap<&str, &dyn ToString>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>,
               ) -> RequestRef 
    {
        let mut owned_params = BTreeMap::new();
        for (key, value) in params {
            owned_params.insert(key.to_string(), value.to_string());
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
    PollParse(Box<dyn Futurev01<Item=T, Error=Error> + Send>),
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
               params: BTreeMap<&str, &dyn ToString>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>,
               ) -> ApiRequest<T>
    {
        let max_attempts = client.config().max_retrys;
        ApiRequest {
            inner: Arc::new(RequestRef::new(url, params, client, method, ratelimit)),
            state: RequestState::SetupRequest,
            attempt: 0,
            max_attempts,
            pagination: None,
        }
    }
}

impl<T: DeserializeOwned + PaginationTrait + 'static + Send> IterableApiRequest<T> {
    
    pub fn new(url: String,
               params: BTreeMap<&str, &dyn ToString>,
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

use crate::sync::barrier::Barrier;
use crate::sync::waiter::Waiter;

struct WaiterState<W: Waiter> {
    polling: bool,
    shared_future: Option<Shared<Box<dyn Futurev01<Item=(), Error=ConditionError> + Send>>>,
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

impl<W: Waiter> Futurev01 for WaiterState<W> {
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
        Shared<Box<dyn Futurev01<Item=(), Error=ConditionError> + Send>> {
        /* If a secret is not provided then just immediately return */
        let secret = self.waiter.secret().unwrap();
        let bottom_client = self.waiter.get_bottom_client();
        let client = self.waiter.clone();

        let auth_future = 
            bottom_client
            .auth()
            .client_credentials(secret)
            .map(move |credentials| {
                if let ClientType::Auth(inner) = client.inner.as_ref() {
                    let mut auth = inner.auth_state.lock().unwrap();
                    auth.state = AuthState::Auth;
                    auth.token = Some(credentials.access_token.clone());
                    if let Some(scopes) = credentials.scope {
                        for scope in scopes { auth.scopes.push(scope) }
                    }
                }
                ()
            })
            .map_err(|err| err.into());

        Futurev01::shared(Box::new(auth_future))
    }
}

impl Waiter for RatelimitWaiter {
    type Item = ();
    type Error = ConditionError;

    fn blocked(&self) -> bool {
        let limits = self.limit.inner.lock().unwrap();
        limits.remaining - limits.inflight <= 0
    }

    fn condition(&self)
        -> Shared<Box<dyn Futurev01<Item=(), Error=ConditionError> + Send>> 
    {
        /*TODO: Really basic for now*/
        use futures_timer::Delay;
        use std::time::Duration;
        let limits = self.limit.clone();
        Futurev01::shared(Box::new(
            Delay::new(Duration::from_secs(60))
            .map(move |_res| {
                let mut limits = limits.inner.lock().unwrap();
                limits.remaining = limits.limit;
                ()
            })
            .map_err(|err| Error::from(err).into())
        ))
    }
}

/* Macro ripped directly from try_ready will retry the connection if any error occurs
 * and there are remaning attempts
 */
#[macro_export]
macro_rules! retry_ready {
    ($s:expr, $e:expr) => (match $e {
        Ok(futuresv01::prelude::Async::Ready(t)) => t,
        Ok(futuresv01::prelude::Async::NotReady) => return Ok(futuresv01::prelude::Async::NotReady),
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
                                max_attempts: self.inner.client.config().max_retrys,
                                pagination: None
                            });
                },
                IterableApiRequestState::PollInner(request) => {
                    let f = request as &mut dyn Futurev01<Item=Self::Item, Error=Self::Error>;
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
                                                    max_attempts: self.inner.client.config().max_retrys,
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

impl <T: DeserializeOwned + 'static + Send> IntoFuture for ApiRequest<T> {

    type Output = Result<T, Error>;
    type Future = Compat01As03<ApiRequest<T>>;

    fn into_future(self) -> Self::Future {
        return Compat01As03::new(self);
    }
}

impl<T: DeserializeOwned + 'static + Send> Futurev01 for ApiRequest<T> {
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
                        trace!("[TWITCH_API] (limit, remaining, inflight) ({:?}, {:?}, {:?})", mut_limits.limit, mut_limits.remaining, mut_limits.inflight);
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
                     

                    let ratelimit_key = self.inner.ratelimit.clone();
                    let client_cloned = client.clone();
                    /*
                    Allow testing by capturing the request and returning a predetermined response
                    If testing is set in the client config then `Pending` is captured and saved and a future::ok(Response) is returned.
                    */
                    let f = 
                        client.send(builder)
                            .then(move |result| {
                                trace!("[TWITCH_API] {:?}", result);
                                if let Some(ratelimit_key) = ratelimit_key {
                                    if let Some(limits) = client_cloned.ratelimit(ratelimit_key) {
                                        let mut mut_limits = limits.inner.lock().unwrap();
                                        mut_limits.inflight = mut_limits.inflight - 1;
                                    }
                                }
                                result
                            })
                            .map_err(|err| err.into())
                            .and_then(|mut response| {
                                let status = response.status();
                                if status.is_success() {
                                    Either::A(
                                        response.json().map_err(|err| Error::from(err)).and_then(|json| {
                                            trace!("[TWITCH_API] {}", json);
                                            serde_json::from_value(json).map_err(|err| err.into())
                                        })
                                    )
                                } else {
                                    Either::B(
                                        response.json::<Message>()
                                        .then(|res| {
                                            match res {
                                                Ok(message) => futuresv01::future::err(Some(message)),
                                                Err(_err) => futuresv01::future::err(None)
                                            }
                                        })
                                        .map_err(move |maybe_message| {
                                            let status = response.status();
                                            if status == 401 || status == 403 {
                                                Error::auth_error(maybe_message)
                                            } else if status == 429 {
                                                Error::ratelimit_error(maybe_message)
                                            } else {
                                                Error::auth_error(maybe_message)
                                            }
                                        })
                                    )
                                }
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
