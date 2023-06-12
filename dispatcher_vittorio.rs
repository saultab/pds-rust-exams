use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use rand::Rng;

struct Dispatcher<Msg: Clone> {
    sender: Mutex<Vec<Sender<Msg>>>,
}

impl<Msg: Clone> Dispatcher<Msg> {
    fn new() -> Dispatcher<Msg> {
            Dispatcher {
                sender: Mutex::new(vec![]),
            }
    }

    fn subscribe(&self) -> Subscription<Msg> {
        let (tx, rx) = channel();
        let mut lock = self.sender.lock().unwrap();
        (*lock).push(tx);
        Subscription::new(rx)
    }

    fn dispatch(&self, msg: Msg) {
        let mut lock = self.sender.lock().unwrap();
        let mut new_vec = vec![];

        for _ in 0..(*lock).len() {
            let tx = (*lock).pop().unwrap();
            new_vec.push(tx.clone());
            tx.send(msg.clone()).unwrap();
        }

        (*lock) = new_vec;
    }
}

struct Subscription<Msg> {
    sub: Receiver<Msg>,
}

impl<Msg: Clone> Subscription<Msg> {
    fn new(rx: Receiver<Msg>) -> Self {
        Subscription {
            sub: rx,
        }
    }

    fn read(&self) -> Option<Msg> {
        let msg = self.sub.recv();
        if msg.is_ok() {
            Some(msg.unwrap())
        } else {
            None
        }
    }
}

fn main() {
    let dispatcher = Dispatcher::new();

    let mut handles = vec![];

    for i in 0..10 {
        handles.push(thread::spawn(
            {
                let sub = dispatcher.subscribe();
                move || {
                    let time = rand::thread_rng().gen_range(0..10);
                    sleep(Duration::from_secs(time));
                    println!("thread {} subscribed!", i);
                    loop {
                        let res = sub.read();
                        match res {
                            None => { break; }
                            Some(msg) => { println!("thread {} received msg {} ", i, msg) }
                        }
                    }
                }
            }
        ))
    }

    for i in 30..35 {
        let time = rand::thread_rng().gen_range(0..2);
        sleep(Duration::from_secs(time));
        dispatcher.dispatch(i);
    }

    drop(dispatcher);

    for h in handles {
        h.join().unwrap();
    }
}
