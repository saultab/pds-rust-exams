use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

const N_THREADS : usize = 3;

struct Sensor{}

impl Sensor{
    fn generate() -> f64{
        let ret: f64 = rand::thread_rng().gen();
        return ret
    }
}

struct Joiner <K: Hash + PartialEq + PartialOrd + Clone + Send, V: Clone + Send>{
    n_threads: usize,
    map_plus_copy_counter: Mutex<(HashMap<K,V>, usize)>,
    cv: Condvar
}

impl<K: Hash + Eq + PartialEq + PartialOrd + Clone + Send, V: Clone + Send> Joiner<K,V>{
    fn new(n_threads: usize) -> Arc<Self> {
        return Arc::new(Joiner{
            n_threads,
            map_plus_copy_counter: Mutex::new((HashMap::new(), 0)),
            cv: Condvar::new()
        })
    }

    fn supply(&self, key: K, value: V) -> HashMap<K,V>{
        let mut lock = self.map_plus_copy_counter.lock().unwrap();
        lock = self.cv.wait_while(lock, |l|{ (*l).1 != 0}).unwrap();

        (*lock).0.insert(key,value);
        if (*lock).0.len() == self.n_threads {self.cv.notify_all()}

        lock = self.cv.wait_while(lock, |l|{ (*l).0.len() <  self.n_threads}).unwrap();

        let ret = (*lock).0.clone();

        (*lock).1 += 1;

        if (*lock).1 == self.n_threads {
            (*lock).1 = 0;
            (*lock).0.clear();
            self.cv.notify_all();
        }

        return ret;
    }
}


fn main() {
    let barrier = Joiner::new(N_THREADS);

    let mut vt = Vec::new();

    for i in 0..N_THREADS {
        vt.push(thread::spawn(
            {
                let b = barrier.clone();
                move || {
                    for _ in 0..5 {
                        let rng: u64 = rand::thread_rng().gen_range(1..5);
                        sleep(Duration::from_secs(rng));

                        let v = Sensor::generate();
                        let map = b.supply(i, v);
                        println!("\nMappa ritornata dal Thread #{i}\n{:?}\n", map);
                    }
                }
            }
        ));
    }

    for t in vt {
        t.join().unwrap();
    }
}