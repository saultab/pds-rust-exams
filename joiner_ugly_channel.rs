use std::collections::HashMap;
use std::fmt::{Display, Debug};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::sleep;
use rand::Rng;
use std::time::Duration;


const N:usize = 3;


struct Joiner<K: Hash + Eq + PartialEq + Clone + Display + Debug + Send + Ord, V: Clone + Display + Debug + Send + Ord > {
    number_threads: usize,
    vec_sender: Mutex<Vec<Sender<i32>>>,
    vec_receiver: Mutex<Vec<Receiver<i32>>>,
    shared_map: Mutex<HashMap<K,V>>
}

impl<K: Hash + Eq + PartialEq + Clone + Display + Debug + Send + Ord, V: Clone + Display + Debug + Send + Ord> Joiner<K, V> {
    fn new(number_threads: usize) -> Self {
        let mut vec_sender = Vec::new();
        let mut vec_receiver = Vec::new();
        for _ in 0..number_threads{
            let (tx,rx) = channel();
            vec_sender.push(tx);
            vec_receiver.push(rx);
        }
        Joiner {
            number_threads,
            vec_sender:Mutex::new(vec_sender),
            vec_receiver: Mutex::new(vec_receiver),
            shared_map: Mutex::new(HashMap::new())
        }
    }

    fn supply(&self, k: K, v: V, index:i32) -> HashMap<K, V> {
        let lock = self.vec_sender.lock().unwrap();
        let senders_vec = lock.clone();
        drop(lock);
        let mut lock = self.vec_receiver.lock().unwrap();
        let receiver = lock.pop().unwrap();
        println!("thread #{index} in scrittura");
        let mut shared = self.shared_map.lock().unwrap();
        shared.insert(k.clone(),v.clone());
        drop(shared);
        drop(lock);
        for i in 0..self.number_threads {
            senders_vec[i].send(1).expect("Error");
        }
        for _ in 0..self.number_threads {
            receiver.recv().expect("Error");
        }
        println!("thread #{index} in copiatura");
        let shared = self.shared_map.lock().unwrap();
        let map = shared.clone();
        drop(shared);
        for i in 0..self.number_threads {
            senders_vec[i].send(1).expect("Error");
        }
        for _ in 0..self.number_threads {
            receiver.recv().expect("Error");
        }
        println!("thread #{index} in calncella");
        let mut shared = self.shared_map.lock().unwrap();
        shared.clear();
        drop(shared);
        for i in 0..self.number_threads {
            senders_vec[i].send(1).expect("Error");
        }
        for _ in 0..self.number_threads {
            receiver.recv().expect("Error");
        }
        println!("map: {:?} / {index}",map);
        let mut lock = self.vec_receiver.lock().unwrap();
        lock.push(receiver);
        drop(lock);
        return map;
    }
}

fn main() {
    let barrier = Arc::new(Joiner::new(N));

    let mut vt = Vec::new();

    for i in 0..N {
        vt.push(std::thread::spawn(
            {
                let b = barrier.clone();
                move || {
                    for _ in 0..3 {
                        let rng: u64 = rand::thread_rng().gen_range(0..2);
                        sleep(Duration::from_secs(rng));
                        b.supply(rng, i, i as i32);
                    }
                }
            }
        ));
    }

    for t in vt {
        t.join().unwrap();
    }
}