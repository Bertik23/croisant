use std::cell::RefCell;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Clock {
    time: Arc<(Mutex<u32>, Condvar)>,
}
pub struct Channel {
    last_time: Arc<RefCell<u32>>,
    time: Arc<(Mutex<u32>, Condvar)>,
}

impl Channel {
    pub fn next(&self) -> u32 {
        let mut t = self.time.0.lock().unwrap();
        while *self.last_time.borrow() == *t {
            t = self.time.1.wait(t).unwrap();
        }
        self.last_time.replace(*t);
        *t
    }
}

impl Clock {
    pub fn new(tick: Duration) -> Clock {
        let time = Arc::new((Mutex::new(0), Condvar::new()));
        {
            let time = time.clone();
            thread::spawn(move || loop {
                thread::sleep(tick);
                let mut t = time.0.lock().unwrap();
                *t += 1;
                time.1.notify_all();
            });
        }
        Clock { time }
    }
    pub fn channel(&self) -> Channel {
        Channel {
            last_time: Arc::new(RefCell::new(*self.time.0.lock().unwrap())),
            time: self.time.clone(),
        }
    }
}

pub struct Ticker;

impl Ticker {
    pub fn new(clock: &Clock, mut f: impl FnMut() + Send + 'static) -> Ticker {
        let cl = clock.clone();
        thread::spawn(move || {
            let c = cl.channel();
            loop {
                c.next();
                f();
            }
        });
        Ticker
    }
}
