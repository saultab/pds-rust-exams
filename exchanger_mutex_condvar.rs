//VITTORIO

use std::fmt::Debug;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use rand::Rng;
use crate::State::{Busy, Wait, Free};

#[derive(PartialEq, Debug)]
enum State {
    Busy,
    Wait,
    Free,
}

struct Exchanger<T> {
    state: Mutex<State>,
    cv: Condvar,
    val1: Mutex<T>,
    val2: Mutex<T>,
}

impl<T: Clone + Debug> Exchanger<T> {
    fn new(v1: T, v2: T) -> Arc<Self> {
        Arc::new(
            Exchanger {
                state: Mutex::new(Free),
                cv: Condvar::new(),
                val1: Mutex::new(v1),
                val2: Mutex::new(v2),
            }
        )
    }

    fn exchange(&self, val: T) -> T {
        let mut lock = self.state.lock().unwrap();

        //lock = self.cv.wait_while(lock, |a| *a != Busy).unwrap();
        while *lock == Busy {
            lock = self.cv.wait(lock).unwrap();
        }

        if *lock == Wait {
            *self.val2.lock().unwrap() = val;
            let ret = (*self.val1.lock().unwrap()).clone();

            *lock = Busy;
            self.cv.notify_all();
            return ret;
        }
        //altrimenti lo stato è Free
        else {
            *lock= Wait;
            *self.val1.lock().unwrap() = val;

            self.cv.notify_one();

            lock = self.cv.wait_while(lock, |a| *a == Wait).unwrap();

            let ret = (*self.val2.lock().unwrap()).clone();
            *lock = Free;
            drop(lock);
            self.cv.notify_one();

            return ret;
        }
    }
}

fn main() {
    let exchanger = Exchanger::new(0, 0);

    let mut vt = Vec::new();

    for i in 0..4 {
        vt.push(thread::spawn({
            let e = exchanger.clone();
            move || {
                let random = rand::thread_rng().gen_range(100..1000);
                println!("Thread #{i} invia {random}");

                let val = e.exchange(random.clone());
                println!("Thread #{i} invia {random} e riceve {val}");
            }
        }
        ));
    }
    for t in vt {
        t.join().unwrap();
    }
}


//MATTEO

/*

use std::fmt::Debug;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::{Rng};

const N_THREADS : usize = 10;

struct Exchanger<T : Debug> {
    values: Mutex<(Option<T>, Option<T>)>,
    cv: Condvar,
}

impl<T : Debug> Exchanger<T>{

    fn new() -> Arc<Self> {
        return Arc::new(Exchanger{
            values : Mutex::new((None,None)),
            cv: Condvar::new()
        })
    }

    fn exchange(&self, value: T) -> T{
        let value_to_return;
        let mut lock = self.values.lock().unwrap();
        lock = self.cv.wait_while(lock, |l| { (*l).1.is_some() }).unwrap();

        if (*lock).0.is_none() {
            println!("{:?} is first", value);
            (*lock).0 = Some(value);
            self.cv.notify_one();
            lock = self.cv.wait_while(lock, |l| { (*l).1.is_none() }).unwrap();
            value_to_return = (*lock).1.take().unwrap();
            self.cv.notify_one();
        } else {
            println!("{:?} is second", value);
            (*lock).1 = Some(value);
            value_to_return = (*lock).0.take().unwrap();
            self.cv.notify_all();
        }

        return value_to_return;
    }
}
*/
