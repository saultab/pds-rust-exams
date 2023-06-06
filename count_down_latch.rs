use std::sync::{Arc, Condvar, Mutex};
use std::thread::sleep;
use std::time::Duration;

struct CountDownLatch {
    counter: Mutex<usize>,
    cv: Condvar,
}

const N : usize = 10;

impl CountDownLatch {
    fn new(limit: usize) -> Arc<Self> {
        Arc::new(CountDownLatch {
            counter: Mutex::new(limit),
            cv: Condvar::new(),
        })
    }

    pub fn count_down(&self, index: usize) {
        println!("Lock é {} in thread #{}", self.counter.lock().unwrap(), index);

        let mut lock = self.counter.lock().unwrap();
        if *lock == 0 { return; }
        *lock -= 1;
        if *lock == 0 {self.cv.notify_all()}
    }

    pub fn my_await(&self, index: usize) {
        let mut lock = self.counter.lock().unwrap();
        while *lock > 0 {
            lock = self.cv.wait(lock).unwrap();
        }
        println!("Lock é 0 in thread #{index}");
    }
}

fn main() {
    let count_down_latch = CountDownLatch::new(N/2);
    let mut vt = Vec::new();

    for i in 0..N {
        vt.push(std::thread::spawn({
            let cdl = count_down_latch.clone();
            move || {
                if i % 2 == 0 {
                    cdl.my_await(i);
                } else {
                    sleep(Duration::from_secs(i as u64));
                    cdl.count_down(i);
                }
            }
        }
        ));
    }
    for t in vt {
        t.join().unwrap();
    }
}
