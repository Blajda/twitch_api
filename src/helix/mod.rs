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
    chan: Option<mpsc::Sender<(AuthWaiter, oneshot::Sender<AuthWaiter>)>>,
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
                        chan: None,
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
                        chan: None,
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

struct RequestRef {
    url: String,
    params: BTreeMap<String, String>,
    client: Client,
    method: Method,
}

enum RequestState<T> {
    Uninitalized,
    WaitAuth(oneshot::Receiver<AuthWaiter>),
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
            }),
            state: RequestState::Uninitalized
        }
    }
}


use futures::Poll;
use serde::de::DeserializeOwned;
use futures::Async;
use futures::try_ready;

/* Consider creating a barrier future which simple takes ownership of the request
 * and returns it after syncing is complete
 */

pub trait Waiter {

    fn is_locked(&self) -> bool;
    fn poll(&self) -> Box<Future<Item=(), Error=()> + Send>;
}

struct AuthWaiter {
    waiter: Client,
}

impl Waiter for AuthWaiter {

    fn is_locked(&self) -> bool {
        let mut_client = self.waiter.inner.inner.lock().unwrap();
        mut_client.auth_state == AuthState::Unauth
    }

    fn poll(&self) -> Box<Future<Item=(), Error=()> + Send> {
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
            .map_err(|err| {
                println!("{:?}", err);
                ()
            });

        Box::new(auth_future)
    }
}

/* Todo: If the polled futures returns an error than all the waiters should
 * get that error
 */

fn create_barrier<T: Send + Waiter + 'static>()
    -> mpsc::Sender<(T, oneshot::Sender<T>)>
{
    enum Message<T> {
        Request((T, oneshot::Sender<T>)),
        OnCondition,
    }
    let (sender, receiver):
        (mpsc::Sender<(T, oneshot::Sender<T>)>, mpsc::Receiver<(T, oneshot::Sender<T>)>) = mpsc::channel(200);

    let mut polling = false;

    let (on_condition_tx, on_condition_rx) = mpsc::unbounded();
    let mut waiters = Vec::new();

    let f1 = receiver.map(|request| Message::Request(request));
    let f2 = on_condition_rx.map(|complete| Message::OnCondition);

    let mut inner_sender = sender.clone();
    let inner_condition = on_condition_tx.clone();
    let f =
        f1.select(f2).for_each(move |message| {
        match message {
            Message::Request((waiter, backchan)) => {
                if waiter.is_locked() && !polling {
                    println!("locked");

                    let c1 = inner_condition.clone();
                    let c2 = inner_condition.clone();
                    let f = waiter
                        .poll()
                        .map(move |_| {c1.send(()).wait(); ()})
                        .map_err(move |_| {c2.send(()).wait(); ()});
                    tokio::spawn(f);
                    polling = true;

                    waiters.push((waiter, backchan));
                } else if waiter.is_locked() || polling {
                    println!("polling");
                    waiters.push((waiter, backchan));
                } else {
                    println!("Pass along waiter!");
                    backchan.send(waiter);
                }
            },
            Message::OnCondition => {
                /*Resubmit all waiters back to the request channel
                 * At least one waiter will pass the barrier
                 */
                polling = false;
                let mut sender = inner_sender.clone();
                while waiters.len() > 0 {
                    let waiter = waiters.pop().unwrap();
                    /* Spawn this */
                    let f = sender.clone().send(waiter);
                    tokio::spawn(f.map(|_| ()).map_err(|_| ()));
                }
            }
        }

        Ok(()) 
    })
    .map(|_| ())
    .map_err(|_| ());

    tokio::spawn(f);

    sender
}


impl<T: DeserializeOwned + 'static + Send> Future for ApiRequest<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            match &mut self.state {
                RequestState::Uninitalized => {
                    let mut mut_client = self.inner.client.inner.inner.lock().unwrap();
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let chan = 
                        mut_client.chan
                        .get_or_insert_with(|| {
                            create_barrier()
                        });

                    /*TODO use poll_ready*/
                    chan.try_send((AuthWaiter{ waiter: self.inner.client.clone() }, resp_tx));

                    self.state = RequestState::WaitAuth(resp_rx);
                },
                RequestState::WaitAuth(chan) => {
                    let waiter = try_ready!(chan.poll());
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
