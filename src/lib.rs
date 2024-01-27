use core::pin::Pin;
use std::{future::Future, time::Duration};


use std::sync::{Arc, Mutex};

mod clock;
use clock::{Clock, Ticker};

pub struct Croissant {
    jobs: Vec<Arc<Mutex<dyn Execute + Send + Sync>>>,
}

struct Job<C> {
    function: Box<dyn Fn(&C) + Send + Sync>,
    context: Box<C>,
}

type AsyncFn<C> = Box<
    dyn (Fn(&C) -> Pin<Box<dyn Future<Output = ()> + Sync + Send>>)
        + Send
        + Sync,
>;

struct AsyncJob<C> {
    function: AsyncFn<C>,
    context: Box<C>,
}

trait Execute {
    fn execute(&self);
}

impl<C> Job<C> {
    fn new(
        function: impl (Fn(&C)) + Send + Sync + 'static,
        context: C,
    ) -> Job<C> {
        Job {
            function: Box::new(function),
            context: Box::new(context),
        }
    }
}

impl<C> AsyncJob<C> {
    fn new(function: AsyncFn<C>, context: C) -> AsyncJob<C>
    where
        C: 'static,
    {
        AsyncJob {
            function: Box::new(function),
            context: Box::new(context),
        }
    }
}

impl<C> Execute for Job<C> {
    fn execute(&self) {
        (*self.function)(&*self.context)
    }
}

impl<C> Execute for AsyncJob<C> {
    fn execute(&self) {
        let fut = (*self.function)(&*self.context);
        tokio::spawn(async move {
            fut.await;
        });
    }
}

impl Croissant {
    pub fn add_job<C>(
        &mut self,
        context: C,
        function: impl Fn(&C) + Send + Sync + 'static,
    ) where
        C: 'static + Send + Sync,
    {
        self.jobs
            .push(Arc::new(Mutex::new(Job::new(function, context))));
    }
    pub fn add_async_job<C>(&mut self, context: C, function: AsyncFn<C>)
    where
        C: 'static + Send + Sync,
    {
        self.jobs
            .push(Arc::new(Mutex::new(AsyncJob::new(function, context))));
    }
    pub fn run_every(&self, step: Duration) {
        let clock = Clock::new(step);
        let jobs = self.jobs.clone();
        let _ticker = Ticker::new(&clock, move || {
            // let g1 = jobs.lock().unwrap();
            jobs.iter()
                .map(|job| {
                    let guard = job.as_ref().lock().unwrap();
                    guard.execute();
                })
                .reduce(|_, _| ());
        });
    }
}
