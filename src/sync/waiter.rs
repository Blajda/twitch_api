use futures::sync::oneshot;
use futures::Future;

pub trait Waiter {
    type Item: Send + 'static;
    type Error: From<Self::ConditionError> 
        + From<oneshot::Canceled> + From<()> + Send + 'static;
    type ConditionError: Send + Clone + 'static;

    fn blocked(&self) -> bool;
    fn condition_poller(&self) -> Box<Future<Item=(), Error=Self::ConditionError> + Send>;
    fn into_future(self) -> Box<Future<Item=Self::Item, Error=Self::Error> + Send>;
}
