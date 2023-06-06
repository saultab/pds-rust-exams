use std::fmt::Error;
use std::fs::canonicalize;
use std::panic::catch_unwind;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use rand::{Rng};

const N: usize = 3;

struct CyclicBarrier {
    sender_to_master: Mutex<Sender<i32>>,
    receivers_threads: Mutex<Vec<Receiver<i32>>>,
    join_handle: JoinHandle<i32>
}

impl CyclicBarrier {
    fn new() -> Arc<Self> {
        let mut senders_to_threads = Vec::new();
        let receivers_threads = Mutex::new(Vec::new());

        for _ in 0..N {
            let (s_t, r_t) = channel();
            senders_to_threads.push(s_t);
            receivers_threads.lock().unwrap().push(r_t);
        }
        let (sender_master, receiver_master) = channel();
        let mut join_handle : JoinHandle<i32> = thread::spawn({
            move || {
                loop {
                    let mut i = 0;
                    while i < N {
                        let res = receiver_master.recv();
                        if res.is_err() { break };

                        i += 1;
                    }

                    for i in 0..N {
                        let res = senders_to_threads[i].send(1);
                        if res.is_err() { break };
                    }
                }
            }
        });


        return Arc::new(CyclicBarrier {
            sender_to_master: Mutex::new(sender_master),
            receivers_threads,
            join_handle
        });
    }

    fn wait(&self, index: usize) {
        let sender_to_master = self.sender_to_master.lock().unwrap().clone();
        let receiver_thread = self.receivers_threads.lock().unwrap().pop().unwrap();

        println!("Thread {} has now stopped!", index);
        sender_to_master.send(1).unwrap();

        //wait
        receiver_thread.recv().unwrap();

        self.receivers_threads.lock().unwrap().push(receiver_thread);
        println!("Thread {} is resuming...", index);

        sleep(Duration::from_secs(rand::thread_rng().gen_range(2..5)));
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
                for _ in 0..3 {
                    c_barrier.wait(index);
                }
            }
        }));
    }

    for h in join_handles {
        h.join().unwrap();
    }
}
