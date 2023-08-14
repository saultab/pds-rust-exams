/*
La classe generica Exchanger<T> permette a due thread di scambiarsi un valore di tipo T. Essa offre esclusivamente il metodo pubblico T exchange( T t) che blocca il
thread chiamante senza consumare CPU fino a che un altro thread non invoca lo stesso metodo, sulla stessa istanza. Quando questo avviene, il metodo restituisce
l’oggetto passato come parametro dal thread opposto.
*/

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::{Rng};

const N_THREADS : usize = 10;

enum State {
    First,
    Second
}

struct Exchanger<T : Debug> {
    channel1: Mutex<(Sender<T>, Receiver<T>)>,
    channel2: Mutex<(Sender<T>, Receiver<T>)>,
    state: Mutex<State>
}

impl<T : Debug> Exchanger<T>{

    fn new() -> Arc<Self> {
        let (s1, r1) = channel();
        let (s2, r2) = channel();
        return Arc::new(Exchanger{
            channel1 : Mutex::new((s1,r2)),
            channel2 : Mutex::new((s2,r1)),
            state: Mutex::new(State::First)
        })
    }

    fn exchange(&self, value: T) -> T{
        let mut lock_state = self.state.lock().unwrap();
        let lock;

         match *lock_state {
            State::First => {
                lock = self.channel1.lock().unwrap();
                *lock_state = State::Second;
            }
            State::Second => {
                lock = self.channel2.lock().unwrap();
                *lock_state = State::First;
            }
        }
        drop(lock_state);

        (*lock).0.send(value).unwrap();
        return (*lock).1.recv().unwrap();
    }
}

fn main() {
    println!("\nwarning: some prints might be out of order\n");
    let exchanger = Exchanger::new();

    let mut vec_join = Vec::new();

    for i in 0..N_THREADS {
        vec_join.push(thread::spawn(
            {
                let e = exchanger.clone();
                move || {
                    let time = rand::thread_rng().gen_range(0..20);
                    sleep(Duration::from_secs(time));
                    println!("thread {} began exchanging procedure", i);
                    let v = e.exchange(i);
                    println!("> thread {} got value {}", i, v);
                }
        }))
    }

    for h in vec_join {
        h.join().unwrap();
    }
}
