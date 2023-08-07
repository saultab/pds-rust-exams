/*
La classe CountDownLatch permette di sincronizzare uno o più thread che devono attendere, senza consumare CPU, il completamento di operazioni in corso in altri thread. All’atto della costruzione, gli oggetti di questa
classe contengono un contatore inizializzato con un valore intero strettamente positivo. 

Oltre al costruttore, questa classe offre due soli metodi pubblici: void await() e void countDown(). Quando un thread invoca await(), rimane bloccato fino a che il contatore non raggiunge il valore 0, dopodiché ritorna; 
se, viceversa, all’atto della chiamata il contatore vale già 0, il metodo ritorna immediatamente.

Quando viene invocato countDown(), se il contatore è maggiore di zero, viene decrementato e se, come conseguenza del decremento, diventa nullo libera i thread bloccati all’interno di await(). Se, viceversa, il contatore
valeva già a zero, l’invocazione di countDown() non ha effetti. 

Si implementi tale classe utilizzando le librerie standard C++11.
*/

use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

const N_THREADS: usize = 10;

struct CountDownLatch{
    cv: Condvar,
    counter: Mutex<usize>
}

impl CountDownLatch {

    fn new(counter: usize) -> Arc<Self> {
        return Arc::new(CountDownLatch{
            cv: Condvar::new(),
            counter: Mutex::new(counter)
        })
    }

    fn awaiting(&self, index: usize) {
        let mut lock = self.counter.lock().unwrap();
        println!("Thread {} is waiting...", index);
        self.cv.wait_while(lock, |l|{*l > 0}).unwrap();
        println!("Thread {} returns!", index);
    }

    fn count_down(&self, index: usize){
        println!("Thread {} counting down...", index);
        let mut lock = self.counter.lock().unwrap();

        if *lock == 0 {return}

        *lock -= 1;
        println!("Thread {} decreased from {} to {}", index, *lock +1, *lock);
        if *lock == 0 {self.cv.notify_all()}
    }
}


fn main() {
    let latch = CountDownLatch::new(N_THREADS /2);

    let mut handles = vec![];

    for i in 0..N_THREADS {
        handles.push(thread::spawn({
            let latch = latch.clone();
            move||{
                let time = rand::thread_rng().gen_range(3..10);
                sleep(Duration::from_secs((time - i % 2 * 2) as u64));
                if i%2 == 0{
                    latch.awaiting(i);
                }
                else{
                    latch.count_down(i);
                }
            }
        }))
    }

    for h in handles {
        h.join().unwrap();
    }
}
