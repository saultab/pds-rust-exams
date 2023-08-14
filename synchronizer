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

use std::fmt::Debug;
use std::sync::Arc;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use rand::Rng;

const N_READS : usize = 10;

//l'implementazione della porta non è richiesta
struct SerialPort{}

impl SerialPort {
    fn new() -> Self {
        return SerialPort{}
    }

    fn read(&self) -> i32 {
        let  time = rand::thread_rng().gen_range(0..5);
        sleep(Duration::from_secs(time));
        return rand::thread_rng().gen_range(0..5);
    }
}

struct Synchronizer<K: Send + 'static + Debug> {
    //quando il synchronizer è instanziato, viene creato un thread aggiuntivo che possiede i capi di entrambi i canali 
    //e si occupa di effettuare il processamento di valori letti
    //quando il synchronizer viene istrutto, bisogna preoccuparsi di terminare il thread e aspettarlo
    //l'attesa è fatta tramite l'implementazione del tratto Drop
    processor_handle: Option<JoinHandle<()>>,
    //sync_channel offre uno strumento di sincronizzazione ready-made synchronization tool, il quale non è solo bloccante per il receiver,
    // ma anche per il sender nel caso il buffer (specificato all creazione del canale) sia pieno
    //tale proprietà rende il canale migliore del classico channel per svolgere questo esercizio senza dover utilizzare condvar per rendere i metodi bloccati
    sender1: Option<SyncSender<K>>,
    sender2: Option<SyncSender<K>>
}

impl<K: Send + 'static + Debug> Synchronizer<K>{
    fn new(process: impl Fn(K,K) -> () + Send + 'static) -> Arc<Self>{
        let (s1, r1) = sync_channel(0);
        let (s2, r2) = sync_channel(0);

        let processor_handle = thread::spawn({
            move||{
                let mut res;
                let mut v1;
                let mut v2;
                loop{
                    //quando i canali sono chiusi la ricezione ritorna un errore e il loop termina
                    //il thread così conclude la propria esecuzione e può essere aspettato dal main 
                    //nel tratto Drop del synchronizer
                    res = r1.recv();
                    match res {
                        Ok(v) => {v1 = v},
                        Err(_) => {break}
                    }

                    res = r2.recv();
                    match res {
                        Ok(v) => {v2 = v},
                        Err(_) => {break}
                    }

                    process(v1, v2);
                }
                println!("Processor is dying...");
            }
        });
        return Arc::new(Synchronizer{
            processor_handle: Some(processor_handle),
            sender1: Some(s1),
            sender2: Some(s2)
        })
    }

    fn data_from_first_port(&self, value: K) {
        self.sender1.clone().unwrap().send(value).unwrap();
    }

    fn data_from_second_port(&self, value: K){
        self.sender2.clone().unwrap().send(value).unwrap();
    }
}

//l'implementazione è necessaria, altrimenti il programma non è totalmente corretto
impl<K: Send + 'static + Debug> Drop for Synchronizer<K> {
    fn drop(&mut self) {
        let s1 = self.sender1.take().unwrap();
        drop(s1);
        let s2 = self.sender2.take().unwrap();
        drop(s2);

        let handle = self.processor_handle.take().unwrap();

        handle.join().unwrap();
    }
}

fn printer<K: Debug>(i1: K, i2: K){
    println!("> Values received = {:?}, {:?}", i1, i2);
}

fn main() {
    let synchronizer = Synchronizer::new(printer);

    //il sistema sfrutta due threads per leggere dalle porte
    let h1 = thread::spawn({
        let s = synchronizer.clone();
        move||{
            let port = SerialPort::new();
            for _ in 0..N_READS {
                let val = port.read();
                println!("sending {val} from port 1");
                s.data_from_first_port(val);
            }
        }
    });

    let h2 = thread::spawn({
        let s = synchronizer.clone();
        move||{
            let port = SerialPort::new();
            for _ in 0..N_READS {
                let val = port.read();
                thread::sleep(Duration::from_secs(4));
                s.data_from_second_port(val);
            }
        }
    });

    h1.join().unwrap();
    h2.join().unwrap();
}
