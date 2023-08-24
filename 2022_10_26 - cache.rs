/*
Un componente con funzionalità di cache permette di ottimizzare il comportamento di un sistema riducendo il numero di volte in cui una funzione è invocata,
tenendo traccia dei risultati da essa restituiti a fronte di un particolare dato in ingresso. Per generalità, si assuma che la funzione accetti un dato di
tipo generico K e restituisca un valore di tipo generico V.

Il componente offre un unico metodo get(...) che prende in ingresso due parametri, il valore k (di tipo K, clonabile) del parametro e la funzione f (di tipo K -> V)
responsabile della sua trasformazione, e restituisce uno smart pointer clonabile al relativo valore.

Se, per una determinata chiave k, non è ancora stato calcolato il valore corrispondente, la funzione viene invocata e ne viene restituito il risultato;
altrimenti viene restituito il risultato già trovato.

Il componente cache deve essere thread-safe perché due o più thread possono richiedere contemporaneamente il valore di una data chiave: quando questo avviene e il dato
non è ancora presente, la chiamata alla funzione dovrà essere eseguita nel contesto di UN SOLO thread, mentre gli altri dovranno aspettare il risultato in corso di
elaborazione, SENZA CONSUMARE cicli macchina.

Si implementi tale componente a scelta nei linguaggi C++ o Rust.
*/

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

const N_THREADS : usize = 5;
const N_KEYS : i32 = 3;


struct Cache<K, V> {
    map: RwLock<HashMap<K, Arc<V>>>
}

impl<K: Display + Clone + Eq + PartialEq + Hash, V: Clone + Display> Cache<K, V> {
    fn new() -> Arc<Self> {
        Arc::new(Cache {
            map: RwLock::new(HashMap::new()),
        })
    }

    fn get(&self, i: usize /*added for debugging only*/, k: K, func: impl Fn(K) -> V) -> Arc<V> {
        let read_lock = self.map.read().unwrap();

        return match read_lock.get(&k) {
            Some(value) => {
                println!("thread #{i} scopre che la chiave {k} esiste già -> ritorno");
                value.clone()
            }
            None => {
                println!("thread #{i} scopre che la chiave {k} NON esiste...");
                drop(read_lock);

                let mut write_lock = self.map.write().unwrap();
                //check if while the thread was waiting to obtain the write permissions
                //another thread has already written
                if write_lock.get(&k).is_some() {
                    println!("thread #{i} scopre che la chiave E' STATA INSERITA MENTRE ASPETTAVA DI INSERIRLA {k} -> ritorno");
                    write_lock.get(&k).unwrap().clone() //by cloning the Arc just the reference is cloned
                }
                else{
                    let val = Arc::new(func(k.clone())); //executed inside lock => avoids calling func more than once per key
                    println!("thread {i} inserisce il valore di f({k})={val}");
                    write_lock.insert(k.clone(),val.clone());
                    val
                }
            }
        }
    }
}

pub fn f<K: Display, V: Display>(k: K) -> V where Standard: Distribution<V> {
    println!(" > Sono dentro la funzione con la chiave...{k}");
    sleep(Duration::from_secs(2));
    let mut rng = rand::thread_rng();
    let val = rng.gen();
    println!(" > la funzione restituisce {k} => {val}");
    val
}

fn main() {
    let cache = Cache::<i32, i32>::new();
    let mut vt = Vec::new();

    for i in 0..N_THREADS {
        vt.push(std::thread::spawn({
            let c = cache.clone();
            move || {
                for _ in 0..N_KEYS {
                    let j = rand::thread_rng().gen_range(0..N_KEYS);
                    let rng : u64 = rand::thread_rng().gen_range(0..5);
                    sleep(Duration::from_secs(rng));
                    println!("thread #{i} con chiave {j} sta per entrare nella get");
                    let ret = c.get(i,j, f);
                    println!("thread #{i} per la chiave {j} restituisce {:?}\n", ret);
                }
            }
        }));
    }
    for v in vt {
        v.join().unwrap();
    }

}
