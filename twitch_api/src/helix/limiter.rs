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

#[derive(Debug, Clone)]
pub struct BucketLimiter(Arc<BucketLimiterInner>);

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

        BucketLimiter(Arc::new(bucket))
    }
}

impl BucketLimiter {
    pub async fn queue(&self, cost: u32) -> Result<(), Error> {
        let f = self.0.inner.until_n_ready(NonZeroU32::new(cost).unwrap());
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
}
