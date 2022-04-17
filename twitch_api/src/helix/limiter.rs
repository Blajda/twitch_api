use crate::error::{Error, Kind};
use governor::{
    clock,
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use hyper::HeaderMap;
use std::time::SystemTime;
use std::{
    num::NonZeroU32,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct BucketLimiter(Arc<Mutex<BucketLimiterInner>>);

#[derive(Debug)]
pub struct BucketLimiterInner {
    inner: RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock, NoOpMiddleware>,
    limit_header: String,
    remaining_header: String,
    reset_header: String,
    limit: u32,
    remaining: u32,
    reset: SystemTime,
}

impl BucketLimiter {
    pub fn new<S: ToString>(
        limit: u32,
        limit_header: &S,
        remaining_header: &S,
        reset_header: &S,
    ) -> BucketLimiter {
        let bucket = BucketLimiterInner {
            limit_header: limit_header.to_string(),
            remaining_header: remaining_header.to_string(),
            reset_header: reset_header.to_string(),
            limit,
            remaining: limit,
            reset: SystemTime::now(),
            inner: RateLimiter::direct(Quota::per_minute(NonZeroU32::new(limit).unwrap())),
        };

        BucketLimiter(Arc::new(Mutex::new(bucket)))
    }
}

impl BucketLimiter {
    pub async fn queue(self, cost: u32) -> Result<(), Error> {
        let limiter = self.0.lock().await;
        let f = limiter.inner.until_n_ready(NonZeroU32::new(cost).unwrap());
        let res = f.await;
        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(Error {
                inner: Kind::RatelimitCostError(
                    "Cost of resouce exceed maximum capacity".to_owned(),
                ),
            }),
        }
    }

    pub async fn update(&self, headers: &HeaderMap) {
        let mut limiter = self.0.lock().await;
        limiter.update_from_headers(headers);
    }
}

impl BucketLimiterInner {
    pub fn update_from_headers(&mut self, headers: &HeaderMap) {
        let maybe_limit = headers
            .get(&self.limit_header)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<u32>().ok());

        if let Some(limit) = maybe_limit {
            self.limit = limit;
        }

        let maybe_remaining = headers
            .get(&self.remaining_header)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<u32>().ok());

        if let Some(limit) = maybe_remaining {
            self.remaining = limit;
        }

        let maybe_reset = headers
            .get(&self.reset_header)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(reset) = maybe_reset {
            self.reset = UNIX_EPOCH + Duration::from_secs(reset);
        }
    }
}
