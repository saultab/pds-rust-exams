/*
Un paradigma frequentemente usato nei sistemi reattivi e costituito dall'astrazione detta Looper.
Quando viene creato, un Looper crea una coda di oggetti generici di tipo Message ed un thread.
II thread attende - senza consumare cicli di CPU - che siano presenti messaggi nella coda,
li estrae a uno a uno nell'ordine di arrivo, e li elabora.

II costruttore di Looper riceve due parametri, entrambi di tipo (puntatore a) funzione: process( ... ) e cleanup().
La prima è una funzione responsabile di elaborare i singoli messaggi ricevuti attraverso la coda;
tale funzione accetta un unico parametro in ingresso di tipo Message e non ritorna nulla;
La seconda e funzione priva di argomenti e valore di ritorno e verra invocata dal thread incapsulato
nel Looper quando esso stara per terminare.

Looper offre un unico metodo pubblico, thread safe, oltre a quelli di servizio, necessari per gestirne ii ciclo di vita:
send(msg), che accetta come parametro un oggetto generico di tipo Message che verra inserito nella coda
e successivamente estratto dal thread ed inoltrato alla funzione di elaborazione.
Quando un oggetto Looper viene distrutto, occorre fare in modo che ii thread contenuto al suo interno
invochi la seconda funzione passata nel costruttore e poi termini.

Si implementi, utilizzando il linguaggio Rust o C++, tale astrazione tenendo conto che i suoi
 metodi dovranno essere thread-safe.
*/

use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::thread::JoinHandle;

struct Looper<Message: Send + Debug + 'static> {
    sender_to_master:Mutex<Option<Sender<Message>>>, //might be unnecessary to use the lock
    master_handle: Option<JoinHandle<()>>
}

impl<Message: Send + Debug + 'static> Looper<Message>{
    fn new(process: impl Fn(Message)->() + Send + 'static, cleanup: impl FnOnce() + Send + 'static) -> Arc<Self>{
    let (s,r) = channel();

    let master_handle = thread::spawn({
        let receiver = r;
        let process = process;
        let cleanup = cleanup;
        move||{
            while let Ok(msg) = receiver.recv() {
                process(msg);
            }
            cleanup();
        }
    });

    return Arc::new(Looper{
        sender_to_master: Mutex::new(Some(s)),
        master_handle: Some(master_handle)
    })
    }

    fn send(&self, msg: Message){
        //i didn't find any smarter way to prevent the bottleneck
        let mut lock = self.sender_to_master.lock().unwrap();
        let sender = (*lock).take().unwrap();
        sender.send(msg).unwrap();
        (*lock) = Some(sender);
    }
}

impl<Message: Send + Debug> Drop for Looper<Message>{
    fn drop(&mut self){
        let sender = self.sender_to_master.lock().unwrap().take().unwrap();
        drop(sender);
        let join_handle = self.master_handle.take().unwrap();
        join_handle.join().unwrap();
    }
}

fn main() {
    let l = Looper::new(process, cleanup);

    let mut vec= vec![];
    for i in 0..5 {
        vec.push(thread::spawn({
            let l_cloned = l.clone();
            move || {
                println!("Sending message #{}", i);
                l_cloned.send(i);
            }
        }))
    }

    for v in vec {
        v.join().unwrap();
    }
}

fn process<Message: Debug>(msg: Message) {
    println!("Processing {:?}", msg);
}

fn cleanup() {
    println!("cleaning...");
}
