use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::sleep;
use rand::Rng;
use std::time::Duration;
use State::{Reading, Writing};

#[derive(Eq, PartialEq)]
enum State {
    Reading,
    Writing,
}

struct Joiner<K: Hash + Eq + PartialEq + Clone + Display, V: Clone + Display> {
    number_threads: usize,
    map: RwLock<HashMap<K, V>>,
    lock: Mutex<(State, usize)>,
    cv: Condvar,
}

impl<K: Hash + Eq + PartialEq + Clone + Display, V: Clone + Display> Joiner<K, V> {
    fn new(number_threads: usize) -> Self {
        Joiner {
            number_threads,
            map: RwLock::new(HashMap::new()),
            lock: Mutex::new((Reading, 0)),
            cv: Condvar::new(),
        }
    }

    fn supply(&self, k: K, v: V) -> HashMap<K, V> {
        let mut lock = self.lock.lock().unwrap();

        println!("Thread #{k} entra in supply...");
        while (*lock).1 > 0 && (*lock).0 == Reading {
            println!("Thread #{k} aspetta che tutti puliscano la propria lettura...");
            lock = self.cv.wait(lock).unwrap();
        }

        if (*lock).0 == Reading {
            //sleep(Duration::from_secs(5));
            println!("Thread #{k} setta lo stato a Writing...");
            (*lock).0 = Writing;
            self.cv.notify_all();
        }

        let mut map_lock = self.map.write().unwrap();
        map_lock.remove(&k);
        drop(map_lock);

        let mut map_lock = self.map.write().unwrap();
        (*map_lock).insert(k.clone(), v.clone());
        println!("--->Thread #{k} insert [{k},{v}]");
        drop(map_lock);

        (*lock).1 += 1;

        if (*lock).1 < self.number_threads {
            while (*lock).0 == Writing {
                println!("Thread #{k} aspetta che tutti scrivino...");
                lock = self.cv.wait(lock).unwrap();
            }
        } else {
            (*lock).0 = Reading;
            self.cv.notify_all();
        }
        drop(lock);

        println!("Thread #{k} legge la mappa...");
        let map_lock = self.map.read().unwrap();
        let map = (*map_lock).clone();
        drop(map_lock);

        //se lasciata qui occorre proteggere condvar o contatore
        // let mut map_lock = self.map.write().unwrap();
        // map_lock.remove(&k);
        // drop(map_lock);

        let mut lock = self.lock.lock().unwrap();
        (*lock).1 -= 1;
        drop(lock);

        println!("Thread #{k} ritorna!");
        return map;
    }
}

fn main() {
    let barrier = Arc::new(Joiner::new(3));

    let mut vt = Vec::new();

    for i in 0..3 {
        vt.push(std::thread::spawn(
            {
                let b = barrier.clone();
                move || {
                    for _ in 0..3 {
                        let rng: u64 = rand::thread_rng().gen_range(3..10);
                        sleep(Duration::from_secs(rng));

                        let map = b.supply(i, rng);
                        println!("\nMappa ritornata dal Thread #{i}");
                        println!("{:?}\n", map);
                    }
                }
            }
        ));
    }

    for t in vt {
        t.join().unwrap();
    }
}
