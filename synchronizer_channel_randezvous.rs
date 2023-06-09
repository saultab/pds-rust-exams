/*
4. Un sistema embedded riceve su due porte seriali sequenze di dati provenienti da due diversi
sensori. Ciascun sensore genera i dati con cadenze variabili nel tempo e non predicibili, in quanto il
processo di digitalizzazione al suo interno può richiedere più o meno tempo in funzione del dato
letto. Ogni volta che il sistema riceve un nuovo valore su una delle due porte seriali, deve
accoppiarlo con il dato più recente ricevuto sull'altra (se già presente) e inviarlo ad una fase
successiva di computazione. Il sistema al proprio interno utilizza due thread differenti per leggere
dalle due porte seriali e richiede l'uso di un oggetto di sincronizzazione in grado di
implementare la logica descritta sopra. Tale oggetto offre la seguente interfaccia pubblica:

class Synchronizer {
    public:
    Synchronizer(std::function<void(float d1, float d2)> process);
    void dataFromFirstPort(float d1);
    void dataFromSecondPort(float d2);
}

All'atto della costruzione, viene fornita la funzione process(...) che rappresenta la fase successiva
della computazione. Quando vengono invocati i metodi dataFromFirstPort(...) o
dataFromSecondPort(...), se non è ancora presente il dato dalla porta opposta, questi si bloccano al
proprio interno senza consumare CPU, in attesa del valore corrispondente. Al suo arrivo, viene
invocata una sola volta la funzione process(...). Si implementi tale classe utilizzando le funzionalità
offerte dallo standard C++.

*/

use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use rand::Rng;

struct SerialPort {
    handle_serial_port: JoinHandle<()>,
}

impl SerialPort {
    fn new(sender_from_sync: SyncSender<f64>) -> Self {
        let handle = thread::spawn(
            move || {
                loop {
                    let time = rand::thread_rng().gen_range(0..5);
                    sleep(Duration::from_secs(time));

                    let read: f64 = rand::thread_rng().gen();
                    let send = sender_from_sync.send(read);
                    if send.is_err() {
                        break;
                    }
                }
            });
        SerialPort {
            handle_serial_port: handle,
        }
    }
}

struct Synchronizer {
    handle_synchronizer: Option<JoinHandle<()>>,
}

impl Synchronizer {
    fn new<F: Fn(f64, f64) -> () + Send + Clone + Copy + 'static>(func: F) -> Self {
        let (tx, rx) = sync_channel(0);

        let handle = thread::spawn(move || {
            let port1 = SerialPort::new(tx.clone());
            let port2 = SerialPort::new(tx.clone());

            for i in 0..10 {
                println!("Lettura #{i}");
                //Miglioria possibile fare in modo che si legga il primo disponibile
                let read1 = rx.recv().unwrap();
                let read2 = rx.recv().unwrap();
                func(read1, read2);
            }

            drop((tx, rx));
            port1.handle_serial_port.join().unwrap();
            port2.handle_serial_port.join().unwrap();
        });
        return Synchronizer {
            handle_synchronizer: Some(handle),
        };
    }
}

impl Drop for Synchronizer {
    fn drop(&mut self) {
        let handle = self.handle_synchronizer.take().unwrap();
        handle.join().expect("Error");
        println!("Tutti a nanna");
    }
}

fn process(r1: f64, r2: f64) {
    println!("sensore 1: {:.3} --- sensore 2: {:.3}", r1, r2);
}

fn main() {
    Synchronizer::new(process);
}