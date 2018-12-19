use super::waiter::Waiter;
use futures::sync::mpsc;
use futures::sync::oneshot;
use futures::prelude::*;

pub trait BarrierSync<W: Waiter> {
    fn wait_for(&mut self, waiter: W) -> Box<Future<Item=W::Item, Error=W::Error> + Send>;
}

pub struct Barrier<W: Waiter> {
    sink: Option<mpsc::Sender<(W, oneshot::Sender<Result<W::Item, W::Error>>)>>,
}

impl<W: Waiter + 'static + Send> BarrierSync<W> for Barrier<W> {
    fn wait_for(&mut self, waiter: W) -> Box<Future<Item=W::Item, Error=W::Error> + Send> {
        let (resp_tx, resp_rx) = oneshot::channel();

        if self.sink.is_none() {
            let (barrier_tx, barrier_rx) = mpsc::channel(40); 
            self.barrier_task(barrier_rx);
            self.sink.replace(barrier_tx);
        }

        let chan = self.sink.as_mut().unwrap().clone();

        /*TODO: I want meaningful error types... */
        let f1 = chan
            .send((waiter, resp_tx))
            .map_err(|err| W::Error::from(()))
            .and_then(|_| {
                resp_rx.then(|result| {
                    match result {
                        Ok(Ok(result)) => Ok(result),
                        Ok(Err(err)) => Err(err),
                        Err(err) => Err(W::Error::from(())),
                    }
                })
            });

        Box::new(f1)
    }
}

impl<W: Waiter + 'static + Send> Barrier<W> {
    pub fn new() -> Barrier<W> {
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
                                let (waiter, backchan) = waiters.pop().unwrap();
                                let f = waiter.into_future()
                                    .then(|res| {
                                        backchan.send(res);
                                        Ok(())
                                    });

                                tokio::spawn(f);
                            }
                        }, 
                        _ => { panic!("condition channel closed") }
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
