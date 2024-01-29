use chrono::NaiveTime;
#[cfg(feature = "async")]
use core::pin::Pin;
#[cfg(feature = "async")]
use std::future::Future;
use std::time::Duration;

use std::sync::{Arc, Mutex};

mod clock;
use clock::{Clock, Ticker};
pub use croissant_macro::croissant;

/// Croissant is a simple task scheduler
/// It can run a task at a specific time or at a specific interval
/// It can run a task in sync or async
/// It is a wrapper around the Ticker struct
/// It is not a cron scheduler, it is a simple scheduler
///
/// Example:
/// ```
/// use croissant::Croissant;
/// use chrono::NaiveTime;
///
/// fn main() {
///     let mut croissant = Croissant::new();
///     croissant.add_job("Hello World", |msg: &str| {
///         println!("{}", msg);
///     });
///     croissant.run_every(Duration::from_secs(5));
///     croissant.run_at(NaiveTime::from_hms(12, 0, 0));
/// }
/// ```
///
/// If you want to use async functions, you need to enable the async feature
/// ```
/// use croissant::Croissant;
/// use chrono::NaiveTime;
/// use std::future::Future;
/// use std::pin::Pin;
///
/// #[croissant]
/// async fn hello(msg: String) {
///     println!("{}", msg);
/// }
///
/// fn main() {
///     let mut croissant = Croissant::new();
///     croissant.add_async_job("Hello World".to_string(), Box::new(hello_croissant));
///     croissant.run_every(Duration::from_secs(5));
///     croissant.run_at(NaiveTime::from_hms(12, 0, 0));
/// }
/// ```
///
/// You can also use the croissant macro for making wrappers
/// around async function, that you can than pass into Croissant
///
///
///
pub struct Croissant {
    jobs: Vec<Arc<Mutex<dyn Execute + Send + Sync>>>,
}

struct Job<C>
where
    C: Clone,
{
    function: Box<dyn Fn(C) + Send + Sync>,
    context: Box<C>,
}

#[cfg(feature = "async")]
type AsyncFn<C> = Box<
    dyn (Fn(C) -> Pin<Box<dyn Future<Output = ()> + Sync + Send>>)
        + Send
        + Sync,
>;

#[cfg(feature = "async")]
struct AsyncJob<C>
where
    C: Clone,
{
    function: AsyncFn<C>,
    context: Box<C>,
}

trait Execute {
    fn execute(&self);
}

impl<C> Job<C>
where
    C: Clone,
{
    fn new(
        function: impl (Fn(C)) + Send + Sync + 'static,
        context: C,
    ) -> Job<C> {
        Job {
            function: Box::new(function),
            context: Box::new(context),
        }
    }
}

#[cfg(feature = "async")]
impl<C> AsyncJob<C>
where
    C: Clone,
{
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

impl<C> Execute for Job<C>
where
    C: Clone,
{
    fn execute(&self) {
        (*self.function)(*self.context.clone())
    }
}

#[cfg(feature = "async")]
impl<C> Execute for AsyncJob<C>
where
    C: Clone,
{
    fn execute(&self) {
        let fut = (*self.function)(*self.context.clone());
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            tokio::spawn(async move {
                fut.await;
            })
        });
    }
}

impl Default for Croissant {
    fn default() -> Self {
        Self::new()
    }
}

impl Croissant {
    /// Creates a new Croissant task scheduler
    pub fn new() -> Self {
        Croissant { jobs: vec![] }
    }
    /// Add a new task to the scheduler
    /// The task will be executed every step
    /// The task will be executed in sync
    ///
    /// Example:
    /// ```
    /// use croissant::Croissant;
    /// use chrono::NaiveTime;
    ///
    /// fn main() {
    ///     let mut croissant = Croissant::new();
    ///     croissant.add_job("Hello World", |msg: &str| {
    ///         println!("{}", msg);
    ///     });
    ///     croissant.run_every(Duration::from_secs(5));
    ///     croissant.run_at(NaiveTime::from_hms(12, 0, 0));
    /// }
    /// ```
    pub fn add_job<C>(
        &mut self,
        context: C,
        function: impl Fn(C) + Send + Sync + 'static,
    ) where
        C: 'static + Send + Sync + Clone,
    {
        self.jobs
            .push(Arc::new(Mutex::new(Job::new(function, context))));
    }
    /// Add a new async task to the scheduler
    /// The task will be executed every step
    /// The task will be executed in async
    ///
    /// Example:
    /// ```
    /// use croissant::Croissant;
    /// use chrono::NaiveTime;
    /// use std::future::Future;
    /// use std::pin::Pin;
    ///
    /// #[croissant]
    /// async fn hello(msg: String) {
    ///     println!("{}", msg);
    /// }
    ///
    /// fn main() {
    ///     let mut croissant = Croissant::new();
    ///     croissant.add_async_job("Hello World".to_string(), Box::new(hello_croissant));
    ///     croissant.run_every(Duration::from_secs(5));
    ///     croissant.run_at(NaiveTime::from_hms(12, 0, 0));
    /// }
    /// ```
    #[cfg(feature = "async")]
    pub fn add_async_job<C>(&mut self, context: C, function: AsyncFn<C>)
    where
        C: 'static + Send + Sync + Clone,
    {
        self.jobs
            .push(Arc::new(Mutex::new(AsyncJob::new(function, context))));
    }
    /// Runs all added jobs every `step`
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
    /// Runs all added jobs every day at `time`
    pub fn run_at(&self, time: NaiveTime) {
        let clock = Clock::start_at(time, Duration::from_secs(60 * 60 * 24));
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

#[cfg(test)]
mod test {
    use crate::{Croissant};
    
    
    use std::{thread, time::Duration};

    #[cfg(feature = "async")]
    async fn job2(_: ()) {
        println!("Job2");
    }

    #[cfg(feature = "async")]
    fn job2_wrapper(
        c: (),
    ) -> Pin<Box<(dyn Future<Output = ()> + Send + Sync)>> {
        Box::pin(job2(c))
    }

    #[cfg(feature = "async")]
    type C = ();

    #[cfg(feature = "async")]
    #[croissant]
    async fn job3(c: C) {
        println!("Job3 {:?}", c)
    }

    #[test]
    fn test() {
        let mut c = Croissant::new();
        c.add_job((), |_| println!("job1"));
        #[cfg(feature = "async")]
        {
            c.add_async_job((), Box::new(job2_wrapper));
            c.add_async_job((), Box::new(job3_croissant));
        }
        // c.run_every(Duration::from_secs(2));
        // c.run_every(Duration::from_secs(1));
        c.run_at(chrono::NaiveTime::from_hms(20, 49, 40));
        thread::sleep(Duration::from_secs(100));
    }
}
