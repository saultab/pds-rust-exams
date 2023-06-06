use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::{Rng};

const N: usize = 3;

struct CyclicBarrier {
    senders: Mutex<Vec<Sender<i32>>>,
    receivers: Mutex<Vec<Receiver<i32>>>,
}

impl CyclicBarrier {
    fn new() -> Arc<Self> {
        let senders = Mutex::new(Vec::new());
        let receivers = Mutex::new(Vec::new());

        for _ in 0..N {
            let (s, r) = channel();
            senders.lock().unwrap().push(s);
            receivers.lock().unwrap().push(r);
        }

        return Arc::new(CyclicBarrier {
            senders,
            receivers,
        });
    }

    fn wait(&self, index: usize) {
        let lock = self.senders.lock().unwrap();
        let senders = lock.clone();
        drop(lock);

        let mut lock = self.receivers.lock().unwrap();
        let receiver = lock.pop().unwrap();
        drop(lock);


        println!("Thread {} has now stopped!", index);
        for s in senders {
            s.send(1).unwrap();
        }

        for _ in 0..N {
            receiver.recv().unwrap();
        }

        let mut lock = self.receivers.lock().unwrap();
        lock.push(receiver);
        drop(lock);
        println!("Thread {} is resuming...", index);


        let t = rand::thread_rng().gen_range(2..5);
        sleep(Duration::from_secs(t));
    }
}

fn main() {
    let a_barrier = CyclicBarrier::new();

    let mut join_handles = vec![];

    for i in 0..N {
        join_handles.push(thread::spawn({
            let c_barrier = a_barrier.clone();
            let index = i.clone();
            move || {
                loop {
                    c_barrier.wait(index);
                }
            }
        }));
    }

    for h in join_handles {
        h.join().unwrap();
    }
}
