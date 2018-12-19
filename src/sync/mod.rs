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
//  2. if we are polling add the request to the queue
//  3. if we are not polling and not locked then 
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

pub mod barrier;
pub mod waiter;
