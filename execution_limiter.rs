use std::sync::{Arc, Condvar, Mutex};
use std::{panic, thread};
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

const N_THREADS : usize = 10;

struct ExecutionLimiter {
    limit: usize,
    counter: Mutex<usize>,
    cv: Condvar
}

impl ExecutionLimiter{
    fn new(n: usize) -> Arc<Self> {
        return Arc::new(ExecutionLimiter{
            limit: n,
            counter: Mutex::new(0),
            cv: Condvar::new()
        })
    }

    fn execute<R: Default >(&self, f: impl Fn() -> R + UnwindSafe ) -> R {
        let mut lock = self.counter.lock().unwrap();
        lock = self.cv.wait_while(lock, |l|{ *l == self.limit}).unwrap();
        (*lock) += 1;
        println!("Starting processing with lock = {}", *lock);
        drop(lock);
        let r = panic::catch_unwind(f);
        let mut lock = self.counter.lock().unwrap();
        (*lock) -= 1;
        println!("Releasing lock... lock = {}", *lock);
        drop(lock);
        self.cv.notify_one();
        return match r {
            Ok(r) => {r}
            Err(_) => {
                println!("panic caught!");
                R::default()
            }
        }
    }
}


fn very_slow_print() -> () {
    let time = rand::thread_rng().gen_range(5..10);
    if time > 8 {   //40% chance of panicking
        println!("PANICKING ='0 ....");
        panic!("oh shush")
    }
    sleep(Duration::from_secs(time));
}

fn main() {
    let execution_limiter = ExecutionLimiter::new(3);

    let mut vec_handles = vec![];

    for i in 0..N_THREADS {
        vec_handles.push(thread::spawn({
            let execution_limiter = execution_limiter.clone();
            move||{
                execution_limiter.execute( very_slow_print);
            }
        }))
    }

    for h in vec_handles {
        h.join().unwrap();
    }

}
