use std::sync::{Arc, Condvar, Mutex};
use std::thread::sleep;
use std::time::Duration;
use crate::State::{Progress, Closure};

#[derive(PartialEq, Debug)]
enum State {
    Progress,
    Closure,
}

struct CyclicBarrier {
    thread_to_wait: usize,
    mutex: Mutex<(usize, State)>,
    condvar: Condvar,
}

impl CyclicBarrier {
    pub fn new(n: usize) -> Self {
        CyclicBarrier {
            thread_to_wait: n,
            mutex: Mutex::new((0, Closure)),
            condvar: Condvar::new(),
        }
    }

    pub fn wait(&self) {

        sleep(Duration::from_secs(1));
        let mut data = self.mutex.lock().unwrap();
        //count = data.0;
        //avanzamento/chiusura = data.1;

        if data.1 == Progress {
            //println!("Stato {:?} --- Count {}", data.1, data.0);
            if data.0 > 0 {
                data.0 -= 1;
                data = self.condvar.wait(data).unwrap();
            }
            if data.0 == 0 {
                data.1 = Closure;
                println!();
                self.condvar.notify_all();
            }
        } else if data.1 == Closure {
            //println!("Stato {:?} --- Count {}", data.1, data.0);
            if data.0 < self.thread_to_wait - 1 {
                data.0 += 1;
                data = self.condvar.wait(data).unwrap();
            }
            else if data.0 == self.thread_to_wait - 1 {
                data.1 = Progress;
                self.condvar.notify_all();
            }
        }
    }
}

fn main() {
    let abarrier = Arc::new(CyclicBarrier::new(10));

    let mut vt = Vec::new();
    for i in 0..10 {
        let cbarrier = abarrier.clone();

        vt.push(std::thread::spawn(move || {
            for j in 0..10 {
                //println!("thread #{} entra", i);
                cbarrier.wait();
                //println!("thread #{} esce", i);
                println!("After barrier thread #{} -> Print {}", i, j);
            }
        }));
    }
    for t in vt {
        t.join().unwrap();
    }
}
