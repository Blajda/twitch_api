use crate::error::{Error, Kind};
use hyper::HeaderMap;
use std::{time::SystemTime, sync::atomic::{AtomicU32, AtomicU64, Ordering, AtomicI32}};
use std::{
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};

#[derive(Debug, Clone)]
pub struct BucketLimiter(Arc<BucketLimiterInner>);

#[derive(Debug)]
pub struct BucketLimiterInner {
    limit_header: String,
    remaining_header: String,
    reset_header: String,
    limit: AtomicI32,
    remaining: AtomicI32,
    inflight: AtomicI32,
    reset: AtomicU64,
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
            limit: AtomicI32::new(limit as i32),
            remaining: AtomicI32::new(limit as i32),
            inflight: AtomicI32::new(0),
            reset: AtomicU64::new(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
        };

        BucketLimiter(Arc::new(bucket))
    }
}

impl BucketLimiter {
    pub async fn take(&self, cost: u32) -> Result<(), Error> {
        let cost = cost as i32;
        if cost > self.0.limit.load(Ordering::Relaxed) {
            return Err(Error {
                inner: Kind::RatelimitCostError(
                    "Cost of resouce exceed maximum capacity".to_owned(),
                ),
            });
        }

    loop {

        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let remaining = self.0.reset.load(Ordering::Relaxed) - now;
        if remaining <= 0 {
            self.0.remaining.store(self.0.limit.load(Ordering::Relaxed), Ordering::Relaxed);
        }

        if cost <= self.0.remaining.load(Ordering::Relaxed) - self.0.inflight.load(Ordering::Relaxed) {
            self.0.inflight.fetch_add(cost, Ordering::Relaxed);
            break;
        }

        let sleep = if remaining > 0 {
            remaining
        } else {
            1
        };
        tokio::time::sleep(Duration::from_secs(sleep)).await;
    }

        Ok(())
    }

    pub async fn restore(&self, cost: u32) -> Result<(), Error> {
        let cost = cost as i32;
        self.0.inflight.fetch_sub(cost, Ordering::Relaxed);
        Ok(())
    }

    pub fn update_from_headers(&self, headers: &HeaderMap) {
        let maybe_limit = headers
            .get(&self.0.limit_header)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<i32>().ok());

        if let Some(limit) = maybe_limit {
            self.0.limit.swap(limit, Ordering::Relaxed);
        }

        let maybe_remaining = headers
            .get(&self.0.remaining_header)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<i32>().ok());

        if let Some(remaining) = maybe_remaining {
            self.0.remaining.swap(remaining,  Ordering::Relaxed);
        }

        let maybe_reset = headers
            .get(&self.0.reset_header)
            .and_then(|x| x.to_str().ok())
            .and_then(|x| x.parse::<u64>().ok());

        if let Some(reset) = maybe_reset {
            self.0.reset.swap(reset, Ordering::Relaxed);
        }
    }
}
