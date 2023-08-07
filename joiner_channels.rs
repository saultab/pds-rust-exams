/*
In una macchina utensile, sono in esecuzione N thread concorrenti, ciascuno dei quali rileva continuamente una sequenza di valori, risultato dell'elaborazione delle 
misurazioni di un sensore. I valori devono essere raggruppati N a N in una struttura dati per essere ulteriormente trattati dal sistema. A questo scopo è definita la 
seguente classe thread-safe: 
    
    class Joiner { 
        public: Joiner(int N); // N is the number of values that must be conferred 
        std::map<int, double> supply(int key, double value);
    }; 
    
Il metodo bloccante supply(...) riceve una coppia chiave/valore generata da un singolo thread e si blocca senza consumare CPU fino a che gli altri N-1 thread hanno inviato 
le loro misurazioni. Quando sono arrivate N misurazioni (corrispondenti ad altrettante invocazioni concorrenti), si sblocca e ciascuna invocazione precedentemente bloccata 
restituisce una mappa che contiene N elementi (uno per ciascun fornitore). Dopodiché, l'oggetto Joiner pulisce il proprio stato e si prepara ad accettare un nuovo gruppo di 
N misurazioni, in modo ciclico. 

Si implementi tale classe, facendo attenzione a non mescolare nuovi conferimenti con quelli della tornata precedente (un thread appena uscito potrebbe essere molto veloce 
a rientrare, ripresentandosi con un nuovo valore quando lo stato non è ancora stato ripulito).
*/



/*
COMMENTO ALLA SOLUZIONE:
Esercizio risolto utilizzando canali. Una soluzione alternativa è inclusa utilizzando mutex + condvar per ottenere la sincronizzazione.
*/


use std::collections::HashMap;
use std::fmt::{Display, Debug};
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use rand::Rng;
use std::time::Duration;

const N_THREADS : usize = 3;

struct Sensor{}

impl Sensor{    //each sensor is assigned to a thread and MUST be associated with a UNIQUE key (see comment at line 66)
fn generate() -> i32{
    let ret: i32 = rand::thread_rng().gen();
    return ret
}
}

struct Joiner<K: Hash + Eq + PartialEq + Clone + Display + Debug + Send + Ord, V: Clone + Display + Debug + Send + Ord> {
    number_threads: usize,
    vec_sender: Mutex<Vec<Sender<(K, V)>>>,
    vec_receiver: Mutex<Vec<Receiver<(K, V)>>>,
}

impl<K: Hash + Eq + PartialEq + Clone + Display + Debug + Send + Ord, V: Clone + Display + Debug + Send + Ord> Joiner<K, V> {
    fn new(number_threads: usize) -> Arc<Self> {
        let mut vec_sender = Vec::new();
        let mut vec_receiver = Vec::new();
        for _ in 0..number_threads {
            let (tx, rx) = channel();
            vec_sender.push(tx);
            vec_receiver.push(rx);
        }
        Arc::new(Joiner {
            number_threads,
            vec_sender: Mutex::new(vec_sender),
            vec_receiver: Mutex::new(vec_receiver),
        })
    }

    fn supply(&self, k: K, v: V) -> HashMap<K, V> {
        let lock = self.vec_sender.lock().unwrap();
        let senders_vec = lock.clone();
        drop(lock);

        let mut lock = self.vec_receiver.lock().unwrap();
        let receiver = lock.pop().unwrap();
        drop(lock);

        let mut map = HashMap::new();

        println!("Thread {k} sending its pair and waiting...");
        for i in 0..self.number_threads {
            senders_vec[i].send((k.clone(), v.clone())).expect("Error");
        }

        let mut dummy = Vec::new(); //this vec is necessary in order to guarantee BOTH ordering and values to remain the same across all maps
        for _ in 0..self.number_threads {
            let (k_rec, v_rec) = receiver.recv().expect("Error");
            dummy.push((k_rec, v_rec));
        }

        dummy.sort();

        for i in 0..self.number_threads {
            let (k_rec, v_rec) = dummy[i].clone();
            map.insert(k_rec, v_rec);
        }

        println!("Thread {k} can now resume!");

        let mut lock = self.vec_receiver.lock().unwrap();
        lock.push(receiver);
        drop(lock);

        return map;
    }
}

fn main() { //main is not required in the exam
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
                        println!("\nMap returned by Thread #{i}\n{:?}\n", map);
                    }
                }
            }
        ));
    }

    for t in vt {
        t.join().unwrap();
    }
}
