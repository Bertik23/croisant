use core::future::Future;
use core::pin::Pin;
use croissant::Croissant;
use std::{thread, time::Duration};
use tokio;

async fn job2() {
    println!("Job2");
}

fn job2_wrapper(_: &()) -> Pin<Box<(dyn Future<Output = ()> + Send + Sync)>> {
    Box::pin(job2())
}

#[tokio::main]
async fn main() {
    let mut c = Croissant::new();
    c.add_job((), |_| println!("job1"));
    c.add_async_job((), Box::new(job2_wrapper));
    // c.run_every(Duration::from_secs(2));
    // c.run_every(Duration::from_secs(1));
    c.run_at(chrono::NaiveTime::from_hms(15, 16, 00));
    thread::sleep(Duration::from_secs(100));
}
