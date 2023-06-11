//VITTORIO

use std::fmt::Debug;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::{Rng};

struct Exchanger<T: Debug> {
    mutex: Mutex<Vec<(Sender<T>, Receiver<T>)>>,
    cv: Condvar,
}

impl<T: Debug> Exchanger<T> {
    fn new() -> Arc<Self> {
        let (s1, r1) = channel();
        let (s2, r2) = channel();

        return Arc::new(Exchanger {
            mutex: Mutex::new(vec![(s1, r2),(s2, r1)]),
            cv: Condvar::new(),
        });
    }

    fn exchange(&self, value: T) -> T {
        let mut lock = self.mutex.lock().unwrap();
        while lock.len() == 0 {
            lock = self.cv.wait(lock).unwrap();
        }

        let ch = (*lock).pop().unwrap();
        drop(lock);

        ch.0.send(value).unwrap();
        let ret = ch.1.recv().unwrap();

        let mut lock = self.mutex.lock().unwrap();
        (*lock).push(ch);

        ret
    }
}

fn main() {
    let exchanger = Exchanger::new();

    let mut vec_join = Vec::new();

    for i in 0..10{
        vec_join.push(thread::spawn(
            {
                let e = exchanger.clone();
                move || {
                    let time = rand::thread_rng().gen_range(0..10);
                    sleep(Duration::from_secs(time));

                    let random = rand::thread_rng().gen_range(100..1000);
                    println!("Thread #{i} invia {random}");

                    let val = e.exchange(random.clone());
                    println!("Thread #{i} invia {random} e riceve {val}");
                }
            }))
    }

    for h in vec_join {
        h.join().unwrap();
    }
}

    
//FRANCESCO
/*
use std::fmt::Display;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;
use rand::Rng;

pub(crate) struct Exchanger<T: Send + Clone + Default + Display> {
    sender1: Mutex<Sender<T>>,
    reciver1: Mutex<Receiver<T>>,
    sender2: Mutex<Sender<T>>,
    reciver2: Mutex<Receiver<T>>,
    val1: Mutex<T>,
    val2: Mutex<T>,
    turn: Mutex<Turn>,
}

enum Turn {
    First,
    Second,
}

impl<T: Send + Clone + Default + Display> Exchanger<T> {
    fn new(v1:T,v2:T) -> Arc<Self> {
        let (tx1, rx1) = channel();
        let (tx2, rx2) = channel();
        Arc::new(Exchanger {
            sender1: Mutex::new(tx1),
            reciver1: Mutex::new(rx2),
            sender2: Mutex::new(tx2),
            reciver2: Mutex::new(rx1),
            val1: Mutex::new(v1),
            val2: Mutex::new(v2),
            turn: Mutex::new(Turn::First),
        })
    }
    fn exchange(&self, val: T) -> T {
        let mut turn = self.turn.lock().unwrap();
        match *turn {
            Turn::First => {
                println!("ehyyy");
                self.sender1.lock().unwrap().send(val).expect("TODO: panic message");
                *turn = Turn::Second;
                drop(turn);
                let val = self.reciver1.lock().unwrap().recv();
                let mut v1 = self.val1.lock().unwrap();
                *v1 = val.clone().unwrap();
                return val.clone().unwrap()
            }
            Turn::Second => {
                println!("ehyyy2");
                self.sender2.lock().unwrap().send(val).expect("TODO: panic message");
                *turn = Turn::First;
                drop(turn);
                let val = self.reciver2.lock().unwrap().recv();
                let mut v2 = self.val2.lock().unwrap();
                *v2 = val.clone().unwrap();
                return val.clone().unwrap()
            }
        }
    }
}

fn main() {
    let exchanger = Exchanger::new(0,0);

    let mut vt = Vec::new();

    for i in 0..2 {
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
*/

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

enum State {
    Free,
    Waiting
}

struct Exchanger<T : Debug> {
    channel1: Mutex<(Sender<T>, Receiver<T>)>,
    channel2: Mutex<(Sender<T>, Receiver<T>)>,
}

impl<T : Debug> Exchanger<T>{

    fn new() -> Arc<Self> {
        let (s1, r1) = channel();
        let (s2, r2) = channel();
        return Arc::new(Exchanger{
            channel1 : Mutex::new((s1,r2)),
            channel2 : Mutex::new((s2,r1)),
        })
    }

    fn exchange(&self, value: T) -> T{
        let mut try_lock = self.channel1.try_lock();

        while try_lock.is_err() {
            try_lock = self.channel2.try_lock();
            if try_lock.is_ok() { break;}
            try_lock = self.channel1.try_lock();
        }

        let mut lock = try_lock.unwrap();

        (*lock).0.send(value).unwrap();
        return (*lock).1.recv().unwrap();
    }
}

fn main() {
    let exchanger = Exchanger::new();

    let mut vec_join = Vec::new();

    for i in 0..N_THREADS {
        vec_join.push(thread::spawn(
            {
                let e = exchanger.clone();
                move || {
                    let time = rand::thread_rng().gen_range(0..20);
                    //sleep(Duration::from_secs(time));
                    let v = e.exchange(i);
                    println!("thread {} got value {}", i, v);
                }
        }))
    }

    for h in vec_join {
        h.join().unwrap();
    }
}
*/
