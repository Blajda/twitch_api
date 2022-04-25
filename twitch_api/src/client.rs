use std::convert::TryFrom;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::Arc;

use std::future::IntoFuture;
use std::task::Poll;

use crate::error::Error;
use crate::helix::limiter::BucketLimiter;
use crate::helix::models::Credentials;
use crate::namespace::auth::client_credentials;
use hyper::body::Body;
use hyper::body::Bytes;
use hyper::client::{Client as HyperClient, HttpConnector};
use hyper::http::response::Parts;
use hyper::Method;
use hyper::Request;
use hyper::{Error as HyperError, Response};
use hyper_tls::HttpsConnector;

use futures::Future;
use std::collections::{HashMap, HashSet};

use serde::de::DeserializeOwned;

use std::collections::BTreeMap;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum RatelimitKey {
    Default,
}

#[derive(Debug)]
pub struct RatelimitMap {
    pub inner: HashMap<RatelimitKey, BucketLimiter>,
}

const API_HELIX_BASE_URI: &str = "https://api.twitch.tv/helix";
const AUTH_BASE_URI: &str = "https://id.twitch.tv/oauth2";

/// Endpoint supports multiple pages of results
pub trait ForwardPagination {
    fn cursor<'a>(&'a self) -> Option<&'a str>;
}

pub struct DefaultOpts {}

/// Endpoint supports multiple pages of results.
/// Can move backwards given the current cursor
pub trait BidirectionalPagination<T> {
    fn next(&self) -> Option<IterableApiRequest<T>>;
    fn prev(&self) -> Option<IterableApiRequest<T>>;
}

/// Internal use only. Used to set attributes for bidirectional pagination
pub trait PaginationContrainerTrait {
    fn set_last_cursor(&mut self, cursor: String);
    fn set_last_direction(&mut self, forward: bool);
    fn set_base_request(&mut self, request: Arc<RequestRef>);
}

pub trait HelixPagination {}

pub type ParamList<'a> = BTreeMap<&'a str, &'a dyn ToString>;

#[derive(Clone, Debug)]
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
    where
        D: Deserializer<'de>,
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
        Ok(match s {
            "analytics:read:extensions" => AnalyticsReadExtensions,
            "analytics:read:games" => AnalyticsReadGames,
            "bits:read" => BitsRead,
            "channel:read:subscriptions" => ChannelReadSubscriptions,
            "clips:edit" => ClipsEdit,
            "user:edit" => UserEdit,
            "user:edit:broadcast" => UserEditBroadcast,
            "user:read:broadcast" => UserReadBroadcast,
            "user:read:email" => UserReadEmail,
            _ => return Err(ScopeParseError {}),
        })
    }
}

#[derive(Clone, Debug)]
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
    pub api_base_uri: String,
    pub auth_base_uri: String,
    pub ratelimits: RatelimitMap,
    pub max_retrys: u32,
}

impl Default for RatelimitMap {
    fn default() -> Self {
        let mut limits = HashMap::new();
        limits.insert(
            RatelimitKey::Default,
            BucketLimiter::new(
                30,
                &"ratelimit-limit",
                &"ratelimit-remaining",
                &"ratelimit-reset",
            ),
        );
        RatelimitMap { inner: limits }
    }
}

impl RatelimitMap {
    pub fn empty() -> RatelimitMap {
        RatelimitMap {
            inner: HashMap::new(),
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
            api_base_uri: API_HELIX_BASE_URI.to_owned(),
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
    scopes: Vec<Scope>,
}

pub trait ClientTrait {
    fn id<'a>(&'a self) -> &'a str;
    fn config<'a>(&'a self) -> &'a ClientConfig;
    fn api_base_uri<'a>(&'a self) -> &'a str;
    fn auth_base_uri<'a>(&'a self) -> &'a str;
    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a BucketLimiter>;

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

    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a BucketLimiter> {
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

    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a BucketLimiter> {
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

    fn ratelimit<'a>(&'a self, key: RatelimitKey) -> Option<&'a BucketLimiter> {
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
    pub fn new<S: Into<String>>(id: S, config: ClientConfig, version: Version) -> Client {
        Client {
            inner: Arc::new(ClientType::Unauth(UnauthClient {
                id: id.into(),
                config: config,
                version: version,
            })),
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
    pub fn new<S: Into<String>>(client: Client, secret: S) -> AuthClientBuilder {
        AuthClientBuilder {
            scopes: HashSet::new(),
            client: client,
            secret: secret.into(),
            token: None,
        }
    }

    pub async fn build(self) -> Result<Client, Error> {
        let old_client = self.client.clone();
        let cred = client_credentials(self.client.clone(), &self.secret).await;
        if let Err(e) = cred {
            return Err(Error::from(e));
        }
        let cred = cred.unwrap();

        Ok(Client {
            inner: Arc::new(ClientType::Auth(AuthClient {
                credentials: cred,
                secret: self.secret,
                previous: old_client,
                scopes: Vec::new(),
            })),
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
    params: Vec<(String, String)>,
    client: Client,
    ratelimit: Option<BucketLimiter>,
    method: Method,
}

impl RequestRef {
    pub fn new(
        url: String,
        params: Vec<(String, String)>,
        client: Client,
        method: Method,
        ratelimit: Option<BucketLimiter>,
    ) -> RequestRef {
        RequestRef {
            url,
            params,
            client,
            method,
            ratelimit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ApiRequest<T> {
    inner: Arc<RequestRef>,
    max_attempts: u32,
    pagination: Option<String>,
    forward: bool,
    _marker: PhantomData<T>,
}

pub struct RequestBuilder<T, Opts = DefaultOpts> {
    url: String,
    params: Vec<(String, String)>,
    client: Client,
    method: Method,
    ratelimit: Option<BucketLimiter>,
    ratelimit_cost: u32,
    _marker: PhantomData<T>,
    _opts: PhantomData<Opts>,
}

impl<T, Opt> RequestBuilder<T, Opt> {
    pub fn with_query<S: Into<String> + ?Sized, S2: Into<String> + ?Sized>(
        mut self,
        key: S,
        value: S2,
    ) -> Self {
        (&mut self.params).push((key.into(), value.into()));
        self
    }

    pub fn with_ratelimit(mut self, bucket: BucketLimiter) -> Self {
        self.ratelimit = Some(bucket);
        self
    }

    pub fn with_ratelimit_cost(mut self, cost: u32) -> Self {
        self.ratelimit_cost = cost;
        self
    }
}

impl<T: DeserializeOwned + ForwardPagination + 'static + Send, Opt> RequestBuilder<T, Opt> {
    pub fn new(client: Client, url: String, method: Method) -> Self {
        RequestBuilder {
            url: url,
            params: Vec::new(),
            ratelimit: (&client)
                .ratelimit(RatelimitKey::Default)
                .map(|m| m.to_owned()),
            client: client,
            ratelimit_cost: 1,
            method: method,
            _marker: PhantomData,
            _opts: PhantomData,
        }
    }

    pub fn build(self) -> ApiRequest<T> {
        ApiRequest::new(
            self.url,
            self.params,
            self.client,
            self.method,
            self.ratelimit,
        )
    }
}

impl<T: DeserializeOwned + ForwardPagination + 'static + Send + HelixPagination, Opt>
    RequestBuilder<T, Opt>
{
    pub fn build_iterable(self) -> IterableApiRequest<T> {
        let r = self.build();
        IterableApiRequest::from_request(&r)
    }
}

impl<T, Opt> IntoFuture for RequestBuilder<T, Opt>
where
    T: DeserializeOwned + ForwardPagination + 'static + Send,
{
    type Output = Result<T, Error>;
    type Future = RequestFuture<T>;

    fn into_future(self) -> Self::Future {
        let request = self.build();
        return request.into_future();
    }
}

pub struct RequestFuture<T> {
    request: ApiRequest<T>,
    state: FutureState,
    _marker: PhantomData<T>,
}

enum FutureState {
    Init,
    PollRateLimit(Pin<Box<dyn Future<Output = Result<(), Error>>>>),
    PollRequest(Pin<Box<dyn Future<Output = Result<Response<Body>, HyperError>>>>),
    PollBody(
        Parts,
        Pin<Box<dyn Future<Output = Result<Bytes, HyperError>>>>,
    ),
}

impl<T> RequestFuture<T> {
    fn build_request_state(&mut self) {
        let request = &self.request;
        let mut query = String::new();
        let mut uri = request.inner.url.clone();

        for (key, value) in &request.inner.params {
            if !query.is_empty() {
                query += "&";
            }
            query = query + key + "=" + value;
        }

        //Add Pagination
        if let Some(page) = &request.pagination {
            let mut key = "after";
            if !query.is_empty() {
                query += "&";
            }
            if !request.forward {
                key = "before"
            }
            query = query + key + "=" + page;
        }

        if !query.is_empty() {
            uri = uri + "?" + &query;
        }

        let mut builder = Request::builder()
            .method(request.inner.method.clone())
            .header("Client-Id", request.inner.client.id())
            .uri(uri);

        if let ClientType::Auth(c) = request.inner.client.inner.as_ref() {
            builder = builder.header(
                "Authorization",
                "Bearer ".to_owned() + &c.credentials.access_token,
            );
        }

        let req = builder.body(Body::empty()).unwrap();
        debug!("{:?}", req);
        let f = request.inner.client.config().hyper.request(req);
        self.state = FutureState::PollRequest(Box::pin(f));
    }
}

impl<T> Future for RequestFuture<T>
where
    T: serde::de::DeserializeOwned,
{
    type Output = Result<T, Error>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        loop {
            match &mut this.state {
                FutureState::Init => match &this.request.inner.ratelimit {
                    Some(limit) => {
                        let l = limit.clone();
                        this.state = FutureState::PollRateLimit(Box::pin(l.queue(1)));
                    }
                    None => {
                        this.build_request_state();
                    }
                },
                FutureState::PollRateLimit(ratelimit) => {
                    let poll = ratelimit.as_mut().poll(cx);
                    match poll {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                        Poll::Ready(Ok(_)) => {
                            this.build_request_state();
                        }
                    }
                }
                FutureState::PollRequest(req) => {
                    let poll = req.as_mut().poll(cx);

                    match poll {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(Error::from(e))),
                        Poll::Ready(Ok(res)) => {
                            let (parts, body) = res.into_parts();
                            this.state =
                                FutureState::PollBody(parts, Box::pin(hyper::body::to_bytes(body)));

                            //TODO: Update Rate limits
                            continue;
                        }
                    };
                }
                FutureState::PollBody(part, body) => {
                    let poll = body.as_mut().poll(cx);

                    match poll {
                        Poll::Pending => return Poll::Pending,
                        Poll::Ready(Err(e)) => return Poll::Ready(Err(Error::from(e))),
                        Poll::Ready(Ok(res)) => {
                            debug!("{:#?}", part);
                            debug!("{:#?}", res);
                            let value = serde_json::from_slice::<T>(res.as_ref());
                            if let Err(e) = value {
                                return Poll::Ready(Err(Error::from(e)));
                            }
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
where
    T: serde::de::DeserializeOwned,
{
    type Output = Result<T, Error>;
    type Future = RequestFuture<T>;

    fn into_future(self) -> Self::Future {
        RequestFuture {
            request: self,
            state: FutureState::Init,
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
    pub fn from_request2(
        request: Arc<RequestRef>,
        cursor: Option<String>,
        forward: bool,
    ) -> IterableApiRequest<T> {
        IterableApiRequest {
            inner: request,
            cursor: cursor,
            _forward: forward,
            _marker: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + ForwardPagination + 'static + Send> ApiRequest<T> {
    pub fn new(
        url: String,
        params: Vec<(String, String)>,
        client: Client,
        method: Method,
        ratelimit: Option<BucketLimiter>,
    ) -> ApiRequest<T> {
        let max_attempts = client.config().max_retrys;
        ApiRequest {
            inner: Arc::new(RequestRef::new(url, params, client, method, ratelimit)),
            max_attempts,
            pagination: None,
            forward: true,
            _marker: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + ForwardPagination + 'static + Send> IterableApiRequest<T> {
    pub fn new(
        url: String,
        params: Vec<(String, String)>,
        client: Client,
        method: Method,
        ratelimit: Option<BucketLimiter>,
    ) -> IterableApiRequest<T> {
        let request_ref = Arc::new(RequestRef::new(url, params, client, method, ratelimit));

        IterableApiRequest {
            inner: request_ref,
            cursor: None,
            _forward: true,
            _marker: PhantomData,
        }
    }
}

pub struct IterableRequestFuture<T> {
    request: Arc<RequestRef>,
    state: IterableApiRequestState<T>,
    _marker: PhantomData<T>,
}

enum IterableApiRequestState<T> {
    PollInner(Pin<Box<RequestFuture<T>>>),
}

impl<T> IntoFuture for IterableApiRequest<T>
where
    T: serde::de::DeserializeOwned + PaginationContrainerTrait,
{
    type Output = Result<T, Error>;
    type Future = IterableRequestFuture<T>;

    fn into_future(self) -> Self::Future {
        let r = self.inner;

        let r = ApiRequest {
            inner: r.clone(),
            max_attempts: 1,
            pagination: self.cursor,
            forward: true,
            _marker: PhantomData,
        };

        IterableRequestFuture {
            request: r.inner.clone(),
            state: IterableApiRequestState::PollInner(Box::pin(r.into_future())),
            _marker: PhantomData,
        }
    }
}

impl<T> Future for IterableRequestFuture<T>
where
    T: serde::de::DeserializeOwned + PaginationContrainerTrait,
{
    type Output = Result<T, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { Pin::get_unchecked_mut(self) };
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
