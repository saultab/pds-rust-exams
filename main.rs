use std::fmt::Debug;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::{Rng};

struct Exchanger<T: Debug> {
    mutex: Mutex<Vec<(Sender<T>, Receiver<T>)>>,
    cv: Condvar,
}

impl<T: Debug> Exchanger<T> {
    fn new() -> Arc<Self> {
        let (s1, r1) = channel();
        let (s2, r2) = channel();

        return Arc::new(Exchanger {
            mutex: Mutex::new(vec![(s1, r2),(s2, r1)]),
            cv: Condvar::new(),
        });
    }

    fn exchange(&self, value: T) -> T {
        let mut lock = self.mutex.lock().unwrap();
        while lock.len() == 0 {
            lock = self.cv.wait(lock).unwrap();
        }

        let ch = (*lock).pop().unwrap();
        drop(lock);

        ch.0.send(value).unwrap();
        let ret = ch.1.recv().unwrap();

        let mut lock = self.mutex.lock().unwrap();
        (*lock).push(ch);

        ret
    }
}

fn main() {
    let exchanger = Exchanger::new();

    let mut vec_join = Vec::new();

    for i in 0..10{
        vec_join.push(thread::spawn(
            {
                let e = exchanger.clone();
                move || {
                    let time = rand::thread_rng().gen_range(0..10);
                    sleep(Duration::from_secs(time));

                    let random = rand::thread_rng().gen_range(100..1000);
                    println!("Thread #{i} invia {random}");

                    let val = e.exchange(random.clone());
                    println!("Thread #{i} invia {random} e riceve {val}");
                }
            }))
    }

    for h in vec_join {
        h.join().unwrap();
    }
}