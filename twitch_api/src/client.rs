use std::convert::TryFrom;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use std::future::IntoFuture;
use std::task::Poll;


use hyper::client::{Client as HyperClient, ResponseFuture, HttpConnector};
use hyper::{Error as HyperError, Response, HeaderMap};
use hyper::Request;
use hyper::Method;
use hyper::body::{HttpBody, Bytes};
use hyper::body::Body;
use hyper::http::response::Parts;
use hyper_tls::HttpsConnector;
use crate::error::Error;
use crate::helix::models::{Credentials, DataContainer, PaginationContainer};
use crate::namespace::auth::client_credentials;

use std::collections::{HashSet, HashMap};
use futures::{Future, FutureExt};

use serde::de::DeserializeOwned;


use std::collections::BTreeMap;


#[derive(PartialEq, Eq, Hash, Clone)]
#[derive(Debug)]
pub enum RatelimitKey {
    Default,
}

#[derive(Debug)]
pub struct RatelimitMap {
    pub inner: HashMap<RatelimitKey, Ratelimit>
}

const API_BASE_URI: &'static str = "https://api.twitch.tv";
const AUTH_BASE_URI: &'static str = "https://id.twitch.tv";

pub trait PaginationTrait {
    fn cursor<'a>(&'a self) -> Option<&'a str>;
}

pub trait PaginationTrait2<T> {
    fn next(&self) -> Option<IterableApiRequest<T>>;
    fn prev(&self) -> Option<IterableApiRequest<T>>;
}

pub trait PaginationContrainerTrait {
    fn set_last_cursor(&mut self, cursor: String);
    fn set_last_direction(&mut self, forward: bool);
    fn set_base_request(&mut self, request: Arc<RequestRef>);
}

pub trait HelixPagination {}

pub type ParamList<'a> = BTreeMap<&'a str, &'a dyn ToString>;

#[derive(Clone)]
#[derive(Debug)]
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
#[derive(Debug)]
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

#[derive(Debug)]
enum ClientType {
    Unauth(UnauthClient),
    Auth(AuthClient),
}


#[derive(Debug)]
pub struct ClientConfig {
    pub hyper: HyperClient<HttpsConnector<HttpConnector>>,
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
        let ratelimits = RatelimitMap::default();
        let https = HttpsConnector::new();
        let hyper = HyperClient::builder().build::<_, Body>(https);

        ClientConfig {
            hyper: hyper,
            api_base_uri: API_BASE_URI.to_owned(),
            auth_base_uri: AUTH_BASE_URI.to_owned(),
            ratelimits,
            max_retrys: 1,
        }
    }
}

#[derive(Debug)]
pub struct UnauthClient {
    id: String,
    config: ClientConfig,
    version: Version,
}

#[derive(Debug)]
pub struct AuthClient {
    credentials: Credentials,
    secret: String,
    previous: Client,
    scopes: Vec<Scope>
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
        true
    }

    fn scopes(&self) -> Vec<Scope> {
        self.scopes.clone()
    }
}

struct AuthStateRef {
    token: Option<String>,
    scopes: Vec<Scope>,
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

    pub async fn build(self) -> Result<Client, Error> {
        let old_client = self.client.clone();
        let cred = client_credentials(self.client.clone(), &self.secret).await;
        if let Err(e) = cred { return Err(Error::from(e)); }
        let cred = cred.unwrap();

        Ok(Client {
            inner: Arc::new(ClientType::Auth(
                AuthClient {
                credentials: cred,
                secret: self.secret,
                previous: old_client,
                scopes: Vec::new(),
            }))
        })
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

#[derive(Debug)]
pub struct RequestRef {
    url: String,
    params: BTreeMap<String, String>,
    client: Client,
    ratelimit: Option<RatelimitKey>,
    method: Method,
}

impl RequestRef {
    pub fn new(url: String,
               params: BTreeMap<String, String>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>,
               ) -> RequestRef 
    {
        /*
        let mut owned_params = BTreeMap::new();
        for (key, value) in params {
            owned_params.insert(key.to_string(), value.to_string());
        }
        */

        RequestRef {
            url: url,
            params: params,
            client: client,
            method: method,
            ratelimit: ratelimit,
        }
    }
}

pub struct ApiRequest<T> {
    inner: Arc<RequestRef>,
    max_attempts: u32,
    pagination: Option<String>,
    forward: bool,
    _marker: PhantomData<T>
}

pub struct RequestBuilder<T> {
    url: String,
    params: BTreeMap<String, String>,
    client: Client,
    method: Method,
    ratelimit_key: Option<RatelimitKey>,
    _marker: PhantomData<T>
}

impl<T: DeserializeOwned + PaginationTrait + 'static + Send> RequestBuilder<T> {

    pub fn new(client: Client, url: String, method: Method) -> Self {
        RequestBuilder {
            url: url,
            params: BTreeMap::new(),
            client: client,
            ratelimit_key: Some(RatelimitKey::Default),
            method: method,
            _marker: PhantomData,
        }
    }

    pub fn build(self) -> ApiRequest<T> {
        ApiRequest::new(self.url, self.params, self.client, self.method, self.ratelimit_key)
    }

    pub fn with_query<S: ToString + ?Sized, S2: ToString +?Sized>(&mut self, key: &S, value: &S2) {
        self.params.insert(key.to_string(), value.to_string());
    }

    pub fn with_ratelimit_key(&mut self, key: RatelimitKey) {
        self.ratelimit_key = Some(key);
    }
}


impl<T: DeserializeOwned + PaginationTrait + 'static + Send + HelixPagination> RequestBuilder<T> {
    pub fn build_iterable(self) -> IterableApiRequest<T> {
        let r = self.build();
        IterableApiRequest::from_request(&r)
    }
}

impl<T> IntoFuture for RequestBuilder<T> 
where T: DeserializeOwned + PaginationTrait + 'static + Send 
{
    type Output = Result<T, Error>;
    type Future = RequestFuture<T>;

    fn into_future(self) -> Self::Future {
        let request = self.build();
        return request.into_future();
    }

}

pub struct RequestFuture<T> {
    state: FutureState,
    _marker: PhantomData<T>,
}

enum FutureState {
    PollRequest(Pin<Box<dyn Future<Output = Result<Response<Body>, HyperError>>>>),
    PollBody(Parts, Pin<Box<dyn Future<Output = Result<Bytes, HyperError>>>>),
}

impl<T> Future for RequestFuture<T> 
    where T : serde::de::DeserializeOwned {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        loop {
            match &mut this.state {
                FutureState::PollRequest(req) => {
                    let poll = req.as_mut().poll(cx);

                    match poll {
                        Poll::Pending => { return Poll::Pending },
                        Poll::Ready(Err(e)) => {return Poll::Ready(Err(Error::from(e)))},
                        Poll::Ready(Ok(res)) => {
                            let (parts, body) = res.into_parts();
                            this.state = FutureState::PollBody(parts, Box::pin(hyper::body::to_bytes(body)));

                            //TODO: Update Rate limits
                            continue;
                        }
                    };
                },
                FutureState::PollBody(part,body) => {
                    let poll = body.as_mut().poll(cx);

                    match poll {
                        Poll::Pending => { return Poll::Pending },
                        Poll::Ready(Err(e)) => {return Poll::Ready(Err(Error::from(e)))},
                        Poll::Ready(Ok(res)) => {
                            debug!("{:#?}", part);
                            debug!("{:#?}", res);
                            let value = serde_json::from_slice::<T>(res.as_ref());
                            if let Err(e) = value { return Poll::Ready(Err(Error::from(e))); }
                            let value = value.unwrap();
                            return Poll::Ready(Ok(value));
                        }
                    };
                }
            }
        }

    }
}

impl<T> IntoFuture for ApiRequest<T> 
where T: serde::de::DeserializeOwned
{
    type Output = Result<T, Error>;
    type Future = RequestFuture<T>;

    fn into_future(self) -> Self::Future {

        let mut query = String::new();
        let mut uri = self.inner.url.clone();


        for (key, value) in &self.inner.params {
            if query.len() > 0 {
                query = query + "&";
            }
            query = query + key + "=" + value;
        }

        //Add Pagination
        if let Some(page) = self.pagination {
            let mut key = "after";
            if query.len() > 0 {
                query = query + "&";
            }
            if !self.forward {
                key = "before"
            }
            query = query + key + "=" + &page;
        }

        if query.len() > 0 {
            uri = uri + "?" + &query;
        }


        let mut builder = Request::builder()
            .method(self.inner.method.clone())
            .header("Client-Id", self.inner.client.id())
            .uri(uri);

        if let ClientType::Auth(c) = self.inner.client.inner.as_ref() {
            builder = builder.header("Authorization", "Bearer ".to_owned() + &c.credentials.access_token);
        }

        let req = builder.body(Body::empty()).unwrap();
        debug!("{:?}", req);
        let f = self.inner.client.config().hyper.request(req);
        RequestFuture {
            state: FutureState::PollRequest(Box::pin(f)),
            _marker: PhantomData,
        }
    }

}


pub struct IterableApiRequest<T> {
    inner: Arc<RequestRef>,
    cursor: Option<String>,
    _forward: bool,
    _marker: PhantomData<T>,
}

impl<T> IterableApiRequest<T> {
    pub fn from_request(request: &ApiRequest<T>) -> IterableApiRequest<T> {
        IterableApiRequest {
            inner: request.inner.clone(),
            cursor: None,
            _forward: true,
            _marker: PhantomData,
        }
    }
    pub fn from_request2(request: Arc<RequestRef>, cursor: Option<String>, forward: bool) -> IterableApiRequest<T> {
        IterableApiRequest {
            inner: request,
            cursor: cursor,
            _forward: forward,
            _marker: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + PaginationTrait + 'static + Send> ApiRequest<T> {

    pub fn new(url: String,
               params: BTreeMap<String, String>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>,
               ) -> ApiRequest<T>
    {
        let max_attempts = client.config().max_retrys;
        ApiRequest {
            inner: Arc::new(RequestRef::new(url, params, client, method, ratelimit)),
            max_attempts,
            pagination: None,
            forward: true,
            _marker: PhantomData
        }
    }
}

impl<T: DeserializeOwned + PaginationTrait + 'static + Send> IterableApiRequest<T> {
    
    pub fn new(url: String,
               params: BTreeMap<String, String>,
               client: Client,
               method: Method,
               ratelimit: Option<RatelimitKey>
               ) -> IterableApiRequest<T>
    {
        let request_ref =
            Arc::new(RequestRef::new(url, params, client, method, ratelimit));

        IterableApiRequest {
            inner: request_ref,
            cursor: None,
            _forward: true,
            _marker: PhantomData,
        }
    }
}

pub struct IterableRequestFuture<T>{
    request: Arc<RequestRef>,
    state: IterableApiRequestState<T>,
    _marker: PhantomData<T>,
}

enum IterableApiRequestState<T> {
    PollInner(Pin<Box<RequestFuture<T>>>),
}

impl<T> IntoFuture for IterableApiRequest<T> 
where T: serde::de::DeserializeOwned + PaginationContrainerTrait
{
    type Output = Result<T, Error>;
    type Future = IterableRequestFuture<T>;

    fn into_future(self) -> Self::Future {
        let r = self.inner;

        let r = ApiRequest {
            inner: r.clone(),
            max_attempts: 4,
            pagination: self.cursor,
            forward: true,
            _marker: PhantomData
        };

        IterableRequestFuture {
            request: r.inner.clone(),
            state:  IterableApiRequestState::PollInner(Box::pin(r.into_future())),
            _marker: PhantomData
        }
    }

}

impl<T> Future for IterableRequestFuture<T> 
    where T : serde::de::DeserializeOwned + PaginationContrainerTrait {
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { Pin::get_unchecked_mut(self)};
        match &mut this.state {
            IterableApiRequestState::PollInner(inner) => {
                let r = inner.as_mut().poll(cx);
                match r {
                    Poll::Pending => Poll::Pending,
                    Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                    Poll::Ready(Ok(mut r)) => {
                        r.set_base_request(this.request.clone());
                        r.set_last_direction(true);
                        Poll::Ready(Ok(r))
                    }   
                }
            }
        }
    }
}

pub struct RatelimitWaiter {
    limit: Ratelimit,
}

#[derive(Clone)]
#[derive(Debug)]
pub struct Ratelimit {
    inner: Arc<Mutex<RatelimitRef>>,
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
    pub fn update_from_headers(&mut self, headers: &HeaderMap) {
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