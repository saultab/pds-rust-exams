use std::sync::{Arc, Condvar, Mutex, RwLock};

const N: usize = 5;

#[derive(PartialEq)]
enum State {
    Progress,
    Closure,
}

struct RankingBarrier {
    n_threads: RwLock<usize>,
    counter: Mutex<(usize,State)>,
    cv: Condvar,
}

impl RankingBarrier {
    fn new(n_threads: usize) -> Result<Arc<Self>, ()> {
        match n_threads {
            0..=1  => {
                Err(())
            }
            _ => {
                Ok(
                    Arc::new(
                        RankingBarrier{
                            n_threads: RwLock::new(n_threads),
                            counter: Mutex::new((0, State::Progress)),
                            cv: Condvar::new(),
                        }
                    )
                )
            }
        }
    }

    pub fn wait(&self) -> usize {
        let mut lock = self.counter.lock().unwrap();
        //self.cv.wait_while( lock, |a| { (*a).1 > 0 } );

        while (*lock).1 == State::Progress && (*lock).0 > 0 {
            lock = self.cv.wait(lock).unwrap();
        }

        if (*lock).1 == State::Progress {
            (*lock).1 = State::Closure;
            println!();
            self.cv.notify_all();
        }
        (*lock).0 += 1;

        let ret = (*lock).0;

        println!("Thread comes {}th", ret);

        while (*lock).0 != *self.n_threads.read().unwrap() && (*lock).1 == State::Closure {
            lock = self.cv.wait(lock).unwrap();
        }

        if (*lock).1 == State::Closure{
            (*lock).1 = State::Progress;
            println!();
            self.cv.notify_all();
        }

        (*lock).0 -= 1;

        println!("Thread returning {}", ret);
        return ret;
    }
}


fn main(){
    let c_barrier = RankingBarrier::new(N).expect("At least 2 threads are required for the barrier to work properly");
    let mut vt = Vec::new();

    for _ in 0..N{
        vt.push(std::thread::spawn({
            let c = c_barrier.clone();
            move || {
                for _ in 0..3 {
                    c.wait();
                }
            }
        }
        ));
    }

    for t in vt {
        t.join().unwrap();
    }
}