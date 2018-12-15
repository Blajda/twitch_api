use std::sync::Arc;
use reqwest::r#async::Client as ReqwestClient;
pub use super::types;

pub mod endpoints;
pub mod models;

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientRef>
}

impl Client {
    pub fn new(id: &str) -> Client {
        Client {
            inner: Arc::new(ClientRef {
                id: id.to_owned(),
                client: ReqwestClient::new(),
            })
        }
    }

    pub fn new_with_client(id: &str, client: ReqwestClient) -> Client {
        Client {
            inner: Arc::new(ClientRef {
                id: id.to_owned(),
                client: client,
            })
        }

    }
}

struct ClientRef {
    id: String,
    client: ReqwestClient,
}

/*

pub struct Limits {
    global: LimiterRef
}

#[derive(Clone)]
pub struct LimiterRef {
    inner: Arc<Mutex<Limiter>>
}

trait RateLimiter {
    fn remaining(&self) -> usize;
    fn limit(&self) -> usize;
}

impl RateLimiter for LimiterRef {

    fn remaining(&self) -> usize {
        let limits = self.inner.lock().unwrap();
        limits.remaining
    }

    fn limit(&self) -> usize {
        let limits = self.inner.lock().unwrap();
        limits.limit
    }
}

struct RequestJob {
    pub request: Request,
    pub on_complete: futures::sync::oneshot::Sender<Response>,
}
*/

/* API requests should be placed in a priority queue to prevent stravation.
 * This implies that all requests are sent a single location and then returned
 * to their callers upon completion.
 * When a request is 'owned' by the queue it can be retryed when the rate limit
 * is hit and allows inspect of response headers to determine remaining resources.
 */

/*

enum Task {
    Add(RequestJob),
    Drain,
}

pub struct Limiter {
    pub remaining: u32,
    pub limit: u32,
        in_transit: u32,
    pub last_request: Option<DateTime<Utc>>,

    pub remaining_key: String,
    pub limit_key: String,

    pub queue: Vec<RequestJob>,
    pub client: ReqwestClient,
    pub chan: mpsc::UnboundedSender<Task>,
}
use futures::sync::oneshot;

fn handle_request(limits_ref: LimiterRef, request: RequestJob) {
    let limits = limits_ref.inner.lock().unwrap();
    limits.queue.push(request);
    limits.chan.unbounded_send(Task::Drain);
}

fn handle_drain(limits_ref: LimiterRef) {

    let jobs = {
        let limits = limits_ref.inner.lock().unwrap();
        let take =
            std::cmp::max(limits.remaining - limits.in_transit, 0);
        let jobs = Vec::new();
        for i in 0..std::cmp::min(limits.queue.len() as u32, take) {
            jobs.push(limits.queue.pop().unwrap());
        }
        limits.in_transit += jobs.len() as u32;
        jobs
    };

    let client = {
        let limits = limits_ref.inner.lock().unwrap();
        &limits.client
    };

    if jobs.len() > 0 {
        for job in jobs {
            let clone = job.request.clone();
            let f = 
                client.execute(job.request)
                .and_then(move |response| {
                    let mut limits = limit_ref.inner.lock().unwrap();
                    limits.in_transit =
                        std::cmp::max(0, limits.in_transit - 1);
                    if response.status().is_success() {
                        let remaining = response.headers()
                            .get(limits.remaining_key)
                            .and_then(|value| value.to_str().ok())
                            .and_then(|remaining| remaining.parse::<usize>().ok());

                        let limit = response.headers()
                            .get(limits.limit_key)
                            .and_then(|value| value.to_str().ok())
                            .and_then(|remaining| remaining.parse::<usize>().ok());

                        if let Some(remaining) = remaining {
                            limits.remaining = remaining;
                        }

                        if let Some(limit) = remaining {
                            limits.limit = limit;
                        }

                        job.on_complete.send(Ok(response));
                    } else if response.status().is_client_error() {
                        limit.chan_tx.send(Handle(
                                RequestJob {
                                    request: clone,
                                    on_complete: job.on_complete.clone(),
                                }))
                        println!("Hit rate limit! or invalid client")
                    }
                                    
}

impl LimiterRef {


    fn handle_drain(&self) {

    }

    fn handle_requests() {

        chan_rx.for_each(move |task| {
            match task {
                Handle(request) => {
                    handle_request( tfdsf, request);                    
                },
                Drain => {
                        }
                    } else {
                        /*sleep...*/
                    }
                }
            }
            Ok(())
        })
        .map(|_| ())
        .map_err(|_| ())
    }


    fn new(limit: u32, remaining_key: &str, limit_key: &str, client: ReqwestClient) 
        -> LimiterRef 
    {
        let (chan_tx, chan_rx) = mpsc::unbounded();
                
        let limiter = Limiter {
            remaining: limit,
            limit: limit,
            in_transit: 0,
            last_request: None,
            remaining_key: remaining_key.to_owned(),
            limit_key: limit_key.to_owned(),
            queue: Vec::new(),
            client: client,
            chan: chan_tx,
        };

        let _ref = LimiterRef {
            inner: Arc::new(Mutex::new(limiter))
        };



        return _ref;
    }

    fn queue(&self, request: Request) 
        -> impl Future<Item=Result<Response, reqwest::Error>, Error=oneshot::Canceled> {
        let mut limits = self.inner.lock().unwrap();
        let limit_ref = self.clone();
        let (tx, rx) = futures::sync::oneshot::channel();

        let job = RequestJob {
            request: request,
            on_complete: tx,
        };

        limits.queue.push(job);
        rx
    }

        /* Insert the request into a queue */
        /*
            Ok(response)
        })
    }
    */

}
*/
