use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::sleep;
use std::time::Duration;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

struct Cache<K, V: Clone> {
    map: RwLock<HashMap<K, V>>,
    executing: Mutex<bool>,
    condvar: Condvar,
}

impl<K: Display + Clone + Eq + PartialEq + Hash, V: Clone> Cache<K, V> {
    fn new() -> Arc<Self> {
        Arc::new(Cache {
            map: RwLock::new(HashMap::new()),
            executing: Mutex::new(false),
            condvar: Condvar::new(),
        })
    }

    fn get(&self, i: i32, k: K, func: impl Fn(K) -> V) -> Arc<V> {
        let read_lock = self.map.read().unwrap();

        match read_lock.get(&k).clone() {
            Some(value) => {
                println!("thread #{i} scopre che la chiave {k} esiste già -> ritorno\n");
                return Arc::new((*value).clone());
            }
            None => {
                println!("thread #{i} scopre che la chiave {k} NON esiste...");
                drop(read_lock);

                let mut lock = self.executing.lock().unwrap();
                while *lock {
                    println!("thread #{i} In attesa...");
                    lock = self.condvar.wait(lock).unwrap();
                }
                *lock = true;

                let read_lock_double_check = self.map.read().unwrap();

                return match read_lock_double_check.get(&k).clone() {
                    Some(value) => {
                        println!("thread #{i} scopre che la chiave {k} esiste già -> ritorno\n");

                        *lock = false;
                        drop(lock);
                        self.condvar.notify_one();

                        Arc::new((*value).clone())
                    }
                    None => {
                        let val = func(k.clone());
                        drop(read_lock_double_check);
                        self.map.write().unwrap().insert(k.clone(), val.clone());
                        println!("-----> thread #{i} ha scritto nella map alla chiave {k}");

                        *lock = false;
                        drop(lock);
                        self.condvar.notify_one();
                        Arc::new(val.clone())
                    }
                };

                //self.map.write().unwrap().insert(k.clone(), val.clone());
            }
        }
    }
}

pub fn f<K: Display, V>(k: K) -> V where Standard: Distribution<V> {
    println!("Sono dentro la funzione con la chiave {k}");
    sleep(Duration::from_secs(2));
    let mut rng = rand::thread_rng();
    let val = rng.gen();
    val
}

fn main() {
    let cache = Cache::<i32, i32>::new();
    let mut vt = Vec::new();

    for i in 0..5 {
        vt.push(std::thread::spawn({
            let c = cache.clone();
            move || {
                for _ in 0..3 {
                    let j = rand::thread_rng().gen_range(0..3);
                    let rng : u64 = rand::thread_rng().gen_range(3..10);
                    sleep(Duration::from_secs(rng));
                    println!("thread #{i} nell'iterazione {j} sta per entrare nella get");
                    c.get(i,j, f);
                    println!("{:?}", c.map);
                }
            }
        }));
    }
    for v in vt {
        v.join().unwrap();
    }

}
