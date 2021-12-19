use futures::Future;
use futures::future::{Shared, SharedError};
use crate::error::ConditionError;

pub trait Waiter {
    type Item: Default;
    type Error: From<SharedError<ConditionError>>; 

    fn blocked(&self) -> bool;
    fn condition(&self) 
        -> Shared<Box<dyn Future<Item=(), Error=ConditionError> + Send>>;
}
