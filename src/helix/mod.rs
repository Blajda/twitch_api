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

struct RequestRef {
    url: String,
    params: BTreeMap<String, String>,
    client: Client,
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

//f.barrier(auth).barrier(ratelimit).and_then(|result| {})
//A ratelimiter must be aware when a limit is hit, the upper limit,
//and remaining requests. (use case specific)
//
//This can be done by either letting the ratelimiter drive the request
//so it can inspect returned headers or by maybe? using a channel to inform 
//the limiter 
//
//Submit task to ratelimiter.
//Check if the limit is hit and if we are polling
//  1  if we hit the limit and are not polling, add to the queue and start
//      polling.
//  1. if we are polling add the request to the queue
//  2. if we are not polling and not locked then 
//      send the request and increment the in-flight counter.
//
//     when the request has completed without errors then decrement
//     the in-flight counter, update limiter data, and return the 
//     result to the requester.
//
//     On error, EITHER: 
//          1. If the error is rate limiter related place the request
//             back in a queue, return other errors. (Prevents starvation)
//          2. Return all errors back to the Requester they can resubmit
//             the request 
//        
// The main difference is that the condition is dependent on the waiter's 
// future result. 
//
// For auth requests we can use an OkFuture that returns the waiter and never errs
//
// So waiters must provide IntoFuture, a future than can poll the condition,
// and a is locked.
// The lock check must be pure (no side effects) but IntoFuture may
// have side effects (eg. increments in-flight counter)
//
//  The result of the IntoFuture is returned to caller or the Err of the poll
//  Future. For simplicity these will be the same type.
//     
//  Should the poll condition trait be located on the Waiter or the Barrier?
//  All waiters in a barrier must use the same condition.

pub trait Waiter {
    type Item: Send + 'static;
    type Error: From<Self::ConditionError> + From<oneshot::Canceled> + Send + 'static;
    type ConditionError: Send + Clone + 'static;

    fn blocked(&self) -> bool;
    fn condition_poller(&self) -> Box<Future<Item=(), Error=Self::ConditionError> + Send>;
    fn into_future(self) -> Box<Future<Item=Self::Item, Error=Self::Error> + Send>;
}

pub trait BarrierSync<W: Waiter> {
    fn wait_for(&mut self, waiter: W) -> Box<Future<Item=W::Item, Error=W::Error> + Send>;
}

pub struct Barrier<W: Waiter> {
    //queue: Vec<(W, oneshot::Sender<Result<W::Item, W::Error>>)>,
    sink: Option<mpsc::Sender<(W, oneshot::Sender<Result<W::Item, W::Error>>)>>,
}

impl<W: Waiter + 'static + Send> Barrier<W> {
    pub fn new() -> Barrier<W> {

        //let f = barrier_rx.for_each(|_| Ok(())).map(|_| ()).map_err(|_| ());
        //tokio::spawn(f);

        Barrier {
            sink: None,
        }
    }

    fn barrier_task(&self, receiver: mpsc::Receiver<(W, oneshot::Sender<Result<W::Item, W::Error>>)>) {

        enum Message<W: Waiter> {
            Request((W, oneshot::Sender<Result<<W as Waiter>::Item, <W as Waiter>::Error>>)),
            OnCondition(Result<(), <W as Waiter>::ConditionError>),
        }

        let mut polling = false;
        let (on_condition_tx, on_condition_rx) = mpsc::unbounded();
        let mut waiters = Vec::new();
        let f1 = receiver.map(|request| Message::Request(request));
        let f2 = on_condition_rx.map(|result| Message::OnCondition(result));

        let inner_condition = on_condition_tx.clone();
        let f =
            f1.select(f2).for_each(move |message| {
            match message {
                Message::Request((waiter, backchan)) => {
                    if waiter.blocked() && !polling {
                        println!("locked");

                        let c1 = inner_condition.clone();
                        let f = waiter
                            .condition_poller()
                            .map(|_| ())
                            .then(|result| {
                                c1.send(result).wait();
                                Ok(())
                            });
                        tokio::spawn(f);
                        polling = true;

                        waiters.push((waiter, backchan));
                    } else if waiter.blocked() || polling {
                        println!("polling");
                        waiters.push((waiter, backchan));
                    } else {
                        println!("Pass along waiter!");
                        //Execute the waiters future//
                        //backchan.send(Ok(waiter));
                        let f = waiter.into_future()
                            .then(|res| {
                                backchan.send(res);
                                Ok(())
                            });

                        tokio::spawn(f);
                    }
                },
                Message::OnCondition(result) => {
                    polling = false;
                    /*Resubmit all waiters back to the request channel
                     * At least one waiter will pass the barrier
                     */
                    match result {
                        Ok(_) => {
                            while waiters.len() > 0 {
                                //Execute the waiters future//
                                //backchan.send(Ok(waiter));
                                let (waiter, backchan) = waiters.pop().unwrap();
                                let f = waiter.into_future()
                                    .then(|res| {
                                        backchan.send(res);
                                        Ok(())
                                    });

                                tokio::spawn(f);
                            }
                        }, 
                        Err(err) => {
                            /*
                            while waiters.len() > 0 {
                                let (waiter, backchan) = waiters.pop().unwrap();
                                backchan.send(Err(<W as Waiter>::Error::from(err.clone())));
                            }
                            */
                        }
                    }
                }
            }



            Ok(()) 
        })
        .map(|_| ())
        .map_err(|_| ());

    tokio::spawn(f);
    }
}

impl<W: Waiter + 'static + Send> BarrierSync<W> for Barrier<W> {
    fn wait_for(&mut self, waiter: W) -> Box<Future<Item=W::Item, Error=W::Error> + Send> {
        let (resp_tx, resp_rx) = oneshot::channel();

        if self.sink.is_none() {
            let (barrier_tx, barrier_rx) = mpsc::channel(40); 
            self.barrier_task(barrier_rx);
            self.sink.replace(barrier_tx);
        }

        let chan = self.sink.as_mut().unwrap();

        /*Clean this up. join it with f2*/
        let f = chan.clone().send((waiter, resp_tx)).map(|_| ()).map_err(|_| ());
        tokio::spawn(f);

        let f2 = resp_rx.then(|result| {
            match result {
                Ok(Ok(result)) => Ok(result),
                Ok(Err(err)) => Err(err),
                Err(err) => Err(W::Error::from(err)),
            }
        });

        Box::new(f2)
    }
}

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
