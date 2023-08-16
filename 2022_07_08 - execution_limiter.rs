/*
All'interno di un programma è necessario garantire che non vengano eseguite CONTEMPORANEAMENTE più di N invocazioni di operazioni potenzialmente lente.
A questo scopo, è stata definita la struttura dati ExecutionLimiter che viene inizializzata con il valore N del limite. 
Tale struttura è thread-safe e offre solo il metodo pubblico generico execute( f ), che accetta come unico parametro una funzione f, priva di parametri 
che ritorna il tipo generico R. Il metodo execute(...) ha, come tipo di ritorno, lo stesso tipo R restituito da f ed ha il compito di mantere il conteggio
di quante invocazioni sono in corso. Se tale numero è già pari al valore N definito all'atto della costruzione della struttura dati, attende, senza provocare 
consumo di CPU, che scenda sotto soglia, dopodiché invoca la funzione f ricevuta come parametro e ne restituisce il valore. Poiché l'esecuzione della funzione f 
potrebbe fallire, in tale caso, si preveda di decrementare il conteggio correttamente. Si implementi, usando i linguaggi Rust o C++, tale struttura dati, 
garantendo tutte le funzionalità richieste.use std::sync::{Arc, Condvar, Mutex};
*/

/*
COMMENTO ALLA SOLUZIONE:
Attenzione: eseguendo il programma saranno riscontrati dei PANIC: essi sono scatenati tramite la macro panic! nella funzione very_slow_print e sono intenzionali, e aderenti alle richieste
*/

use std::sync::{Arc, Condvar, Mutex};
use std::{panic, thread};
use std::fmt::{Display, Formatter};
use std::panic::{UnwindSafe};
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

const N_THREADS : usize = 10;
const PANIC_CHANCE : f32 = 0.4;

struct ExecutionLimiter{
    limit: usize,
    executions: Mutex<usize>,
    cv: Condvar
}

impl ExecutionLimiter{
    fn new(limit: usize) -> Arc<Self>{
        return Arc::new(ExecutionLimiter{
            limit,
            executions: Mutex::new(0),
            cv: Condvar::new()
        })
    }

    fn execute<R>(&self, index: usize, f: impl FnOnce() -> R + UnwindSafe ) -> Result<R, ExecutionError> {
        let mut lock = self.executions.lock().unwrap();
        lock = self.cv.wait_while(lock, |l|{*l == self.limit}).unwrap();
        *lock +=1;

        println!("Thread {} is inside: #threads in execution = {}", index, *lock);
        drop(lock);

        let res = panic::catch_unwind(f);

        println!("Thread {} ended execution", index);

        *(self.executions.lock().unwrap()) -= 1;
        self.cv.notify_one();

        return match res {
            Ok(value) => {Ok(value)},
            Err(_) => {Err(ExecutionError::new())}
        }

    }
}

struct ExecutionError;

impl ExecutionError{
    fn new() -> Self {return ExecutionError{}}
}

impl Display for ExecutionError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Function passed into ExecutionLimiter::execute() ended with panic!")
    }
}


fn very_slow_print() -> i32 {
    let time = rand::thread_rng().gen_range(5..10);
    if time > 5 + ((5 as f32)*PANIC_CHANCE) as u64  {   //40% chance of panicking
        println!("PANICKING....");
        panic!("oh shush");
    }
    sleep(Duration::from_secs(time));
    return 0
}

fn main() {
    let execution_limiter = ExecutionLimiter::new(3);

    let mut vec_handles = vec![];

    for i in 0..N_THREADS {
        vec_handles.push(thread::spawn({
            let execution_limiter = execution_limiter.clone();
            move||{
               // println!("Executing thread {}", i);
                let _res = execution_limiter.execute(i, very_slow_print);
                /*match _res {
                    Ok(v) => {println!("Thread {} ended with status {}", i, v)},
                    Err(e) => {println!("Thread {}: {}",i, e)},
                }*/
            }
        }))
    }

    for h in vec_handles {
        h.join().unwrap();
    }

}
