use super::waiter::Waiter;
use futures::prelude::*;
use std::sync::{Arc, Mutex};
use futures::future::{Shared, SharedError};

use crate::error::ConditionError;

#[derive(Clone)]
pub struct Barrier {
    inner: Arc<Mutex<BarrierRef>>,
}

struct BarrierRef {
    condition: Option<Shared<Box<Future<Item=(), Error=ConditionError> + Send>>>
}

impl Barrier {

    pub fn new() -> Barrier {
        Barrier {
            inner: Arc::new(Mutex::new(
            BarrierRef {
                condition: None,
            }))
        }
    }

    pub fn condition(&self, waiter: &impl Waiter) 
        -> Shared<Box<Future<Item=(), Error=ConditionError> + Send>> 
    {
        let mut mut_barrier = self.inner.lock().unwrap();
        let maybe_condition = &mut mut_barrier.condition;

        let f = maybe_condition.get_or_insert_with(|| {
            waiter.condition()
        });

        let f =
            if let Some(_) = f.peek() {
                let condition = waiter.condition();
                maybe_condition.replace(condition);
                maybe_condition.as_ref().unwrap()
            } else { f };
        f.clone()
    }
}

