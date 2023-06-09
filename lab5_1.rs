use std::sync::{Arc, Condvar, mpsc, Mutex};
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use rand::{Rng};


/*
            STEP #1 -> Scheduler send to all Workers
        ┌──────────────────────────────────────────────┐
        │ Rx channel to receive ptr of Function()      │
        ├──────────────────────────────────────────────┤
        │ Rx channel to receive free state info        │
        ├──────────────────────────────────────────────┤
        │ Tx channel to allow the worker to deliver    │
        │ free state info                              │
        ├──────────────────────────────────────────────┤
        │ index of worker                              │
        └──────────────────────────────────────────────┘

        STEP #2 -> Scheduler send to all Workers jobs and sleep...

                  Scheme of channel to send JOB
                      ┌───> W
                      │
        Scheduler ────┼───> W
                      │
                      └───> W

        STEP #3 -> Worker #1 send to Scheduler own index to set free state

                  Scheme of channel to receive free state info
                       ┌───  W #1
                  <────┘
        Scheduler <────────| W #2
                  <────┐
                       └───| W #3
*/

static N: usize = 3;

struct Worker {
    _handle: JoinHandle<()>,
}

impl Worker {
    fn new<F: FnOnce() -> () + Send + Clone + 'static>(from_scheduler: Receiver<F>,
                                                       to_scheduler: Sender<usize>,
                                                       index: usize) -> Self {
        let handle = thread::spawn({
            move || {
                loop {
                    let result_task = from_scheduler.recv();
                    if result_task.is_err() { break; }
                    let task = result_task.unwrap();
                    task();

                    to_scheduler.send(index).unwrap();
                    println!("Worker #{} finishing is job", index);
                }
            }
        });

        Worker { _handle: handle }
    }
}

struct ThreadPool<F: FnOnce() -> () + Send + Clone + 'static> {
    to_scheduler: Option<Sender<F>>,
    scheduler_handle: Option<JoinHandle<()>>,
}


impl<F: FnOnce() -> () + Send + Clone + 'static> ThreadPool<F> {
    fn new(n_threads: usize) -> Self {
        let (to_scheduler_from_main, from_main): (Sender<F>, Receiver<F>) = mpsc::channel();

        // Thread Pool spawna un thread scheduler
        let scheduler_handle = thread::spawn(move || {
            //println!("Scheduler is running!!!");

            // Mutex tracking number of free workers
            let mut free_workers = n_threads;

            // Channel to receive index of workers freeing themselves
            let (to_scheduler_from_worker, from_worker): (Sender<usize>, Receiver<usize>) = mpsc::channel();

            let mut free_workers_table = vec![];        //1 Table to track free workers
            let mut to_workers_senders: Vec<Sender<F>> = vec![];  //2 Senders vec from scheduler to workers
            let mut workers_vec = vec![];             //3 Worker vec for join handles

            for i in 0..n_threads {
                free_workers_table.push(true); //1

                let (s, r) = channel();
                to_workers_senders.push(s); //2

                workers_vec.push(   //3
                                    Worker::new(
                                        r,
                                        to_scheduler_from_worker.clone(),
                                        i));
            }

            let mut finished = false;
            loop {
                //println!("Reading from main...");
                let task = from_main.recv();
                match task {
                    Ok(_) => {}
                    Err(_) => {
                        //println!("finished");
                        finished = true;
                        if free_workers == n_threads {
                            break;
                        }
                    }
                }

                if free_workers == 0 || finished {
                    //println!("Attendo worker...");
                    let index = from_worker.recv().unwrap();  // Aspetto un messaggio di free state da un worker
                    free_workers_table[index] = true;               // Aggiorno a true l'indice della worker table
                    free_workers += 1;
                    //println!("Worker {} terminated", index);
                }
                if !finished {
                    for (i, w) in free_workers_table.iter_mut().enumerate() {
                        match w {
                            true => {
                                *w = false;
                                free_workers -= 1;
                                println!("Assing job to worker {}", i);
                                to_workers_senders[i].send(task.clone().unwrap().clone()).unwrap();
                                break;
                            }
                            false => {}
                        }
                        //println!("Free Worker: {:?}",free_workers_table);
                    }
                }
            }

            drop(to_workers_senders);

            for w in workers_vec {
                w._handle.join().unwrap();
            }
        });

        return ThreadPool {
            to_scheduler: Some(to_scheduler_from_main),
            scheduler_handle:
            Some(scheduler_handle),
        };
    }

    fn execute(&self, task: F) {
        //invia il task allo scheduler
        self.to_scheduler.clone().unwrap().send(task).expect("Error sending to scheduler");
    }
}

impl<F: FnOnce() -> () + Send + Clone + 'static> Drop for ThreadPool<F> {
    fn drop(&mut self) {
        println!("dropping thread_pool");
        let sender = self.to_scheduler.take().unwrap();
        drop(sender);
        let handle = self.scheduler_handle.take().unwrap();
        handle.join().unwrap();
    }
}

fn consume() {
    let time = rand::thread_rng().gen_range(0..1);
    sleep(Duration::from_secs(time));
    //println!("Task Done \n");
}

fn main() {
    let pool = ThreadPool::new(N);

    for _ in 0..15 {
        pool.execute(consume);
    }
}