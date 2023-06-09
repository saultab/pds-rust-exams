use std::sync::{mpsc};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::{JoinHandle, sleep};
use crossbeam::channel;
use rand::Rng;
use std::time::{Duration, Instant};

/*
            STEP #1 -> Scheduler send to all Workers
        ┌──────────────────────────────────────────────┐
        │ Rx channel to receive ptr of Function()      │
        └──────────────────────────────────────────────┘

        STEP #2 -> Scheduler send on shared channel with ALL WORKER the jobs and after it end...

                  Scheme of channel SHARED to send JOB
                             ┌───> W
                             │
        Scheduler ───> .....─┼───> W
                             │
                             └───> W

        STEP #3 -> Workers take a job if there

*/
static NUMBER_OF_WORKERS: usize = 3;

struct Worker {
    worker_handle: JoinHandle<()>,
    index: usize,
}

impl Worker {
    fn new<F: FnOnce() -> () + Send + Clone + 'static>(from_scheduler: channel::Receiver<F>, index: usize) -> Self {
        let handle = thread::spawn({
            move || {
                loop {
                    let result_taking = from_scheduler.recv();
                    if result_taking.is_err() {
                        break;
                    }
                    let job = result_taking.unwrap();
                    job();
                    println!("Worker #{index} finishing his job");
                }
            }
        });
        Worker { worker_handle: handle, index }
    }
}

struct ThreadPool<F: FnOnce() -> () + Send + 'static> {
    to_scheduler: Option<Sender<F>>,
    scheduler_handle: Option<JoinHandle<()>>,
}

impl<F: FnOnce() -> () + Send + 'static + Clone> ThreadPool<F> {
    fn new() -> Self {

        //Channel mpsc from execute to scheduler
        let (to_scheduler, from_execute) = mpsc::channel();

        let scheduler_handle = thread::spawn({
            move || {

                //Worker vec for join handles
                let mut workers_vec = vec![];

                //Creo un canale MPMC nella quale inserire N jobs
                let (tx, rx): (channel::Sender<F>, channel::Receiver<F>) = channel::unbounded();

                //creo N Worker che preleveranno i jobs dal canale e pusho la handle
                for i in 0..NUMBER_OF_WORKERS {
                    workers_vec.push(Worker::new(rx.clone(), i));
                }
                let mut i = 1;
                loop {
                    println!("Reading from execute... {}", i);
                    i += 1;
                    let job = from_execute.recv();

                    if job.is_err() {
                        println!("Scheduler ending....");
                        break;
                    } else {
                        tx.send(job.unwrap()).expect("Error send");
                    }
                }

                drop((tx, rx));
                for w in workers_vec {
                    w.worker_handle.join().unwrap();
                }
            }
        });

        return ThreadPool {
            to_scheduler: Some(to_scheduler),
            scheduler_handle: Some(scheduler_handle),
        };
    }

    fn execute(&self, job: F) {
        //invia il task allo scheduler
        self.to_scheduler.clone().unwrap().send(job).expect("Error sending to scheduler");
    }
}

impl<F: FnOnce() -> () + Send + 'static> Drop for ThreadPool<F> {
    fn drop(&mut self) {
        println!("dropping thread_pool");
        let sender = self.to_scheduler.take().unwrap();
        drop(sender);
        let handle = self.scheduler_handle.take().unwrap();
        println!("Job esauriti dalla execute e main ha già droppato la sua estremità tx del canale...");
        handle.join().unwrap();
    }
}

fn consume() {
    let time = rand::thread_rng().gen_range(0..3);
    sleep(Duration::from_secs(time));
    //println!("Task Done");
}

fn main() {
    let pool = ThreadPool::new();

    for _ in 0..15 {
        pool.execute(consume);
    }
}
