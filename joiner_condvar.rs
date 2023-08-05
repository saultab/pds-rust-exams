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
Esercizio risolto utilizzando mutex + condvar. Una soluzione alternativa è inclusa utilizzando canali per ottenere la sincronizzazione.
*/
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
