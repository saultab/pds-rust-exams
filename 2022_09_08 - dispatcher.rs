/*
In un sistema concorrente, ciascun thread può pubblicare eventi per rendere noto ad altri thread quanto sta facendo. 
Per evitare un accoppiamento stretto tra mittenti e destinatari degli eventi, si utilizza un Dispatcher: questo è un oggetto thread-safe che offre il metodo

        dispatch(msg: Msg)
        
mediante il quale un messaggio di tipo generico Msg (soggetto al vincolo di essere clonabile) viene reso disponibile a chiunque si sia sottoscritto. 
Un thread interessato a ricevere messaggi può invocare il metodo

        subscribe()

del Dispatcher: otterrà come risultato un oggetto di tipo Subscription mediante il quale potrà leggere i messaggi che da ora in poi saranno pubblicati attraverso 
il Dispatcher. Per ogni sottoscrizione attiva, il Dispatcher mantiene internamente l'equivalente di una coda ordinata (FIFO) di messaggi non ancora letti. 
A fronte dell'invocazione del metodo dispatch(msg:Msg), il messaggio viene clonato ed inserito in ciascuna delle code esistenti. L'oggetto Subscription offre il 
metodo bloccante

        read() -> Option<Msg>

se nella coda corrispondente è presente almeno un messaggio, questo viene rimosso e restituito; se nella coda non è presente nessun messaggio e il Dispatcher esiste 
ancora, l'invocazione si blocca fino a che non viene inserito un nuovo messaggio; se invece il Dispatcher è stato distrutto, viene restituito il valore corrispondente 
all'opzione vuota. 

Gli oggetti Dispatcher e Subscription sono in qualche modo collegati, ma devono poter avere cicli di vita indipendenti: la distruzione del Dispatcher non deve impedire la 
consumazione dei messaggi già recapitati ad una Subscription, ma non ancora letti; parimenti, la distruzione di una Subscription non deve impedire al Dispatcher di 
consegnare ulteriori messaggi alle eventuali altre Subscription presenti. 

Si implementino le strutture dati Dispatcher e Subscription, a scelta, nel linguaggio Rust o C++11.
*/

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
