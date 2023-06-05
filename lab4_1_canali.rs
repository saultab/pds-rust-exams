use std::sync::mpsc::{channel, Receiver, Sender};

static NUMBER_OF_THREAD: usize = 3;

struct CyclicBarrier {
    counter: usize,
    sender: Vec<Sender<usize>>,
    receiver: Receiver<usize>,
}

impl CyclicBarrier {
    pub fn new(sender: Vec<Sender<usize>>, receiver: Receiver<usize>) -> Self {
        CyclicBarrier {
            counter: 0,
            sender,
            receiver,
        }
    }

    pub fn wait(&mut self) {

        //Forward tokens
        for i in 0..NUMBER_OF_THREAD {
            self.sender[i].send(1).expect("Error forward token");
        }

        //Wait the tokens of other threads
        for _ in 0..NUMBER_OF_THREAD {
            self.counter += self.receiver.recv().unwrap();
        }

        //Reset counter of tokens
        self.counter = 0;
    }
}

fn main() {
    let mut vec_channel_receiver = Vec::<Receiver<usize>>::new();
    let mut vec_channel_sender = Vec::<Sender<usize>>::new();

    // Create a vec of channels for each thread
    for _ in 0..NUMBER_OF_THREAD {
        let (sender, receiver) = channel();
        vec_channel_sender.push(sender);
        vec_channel_receiver.push(receiver);
    }
    let mut vt = Vec::new();

    //Cycle from #threads to zero because pop is the only method that takes ownership
    for i in (0..NUMBER_OF_THREAD).rev() {
        vt.push(std::thread::spawn({
            let mut cbarrier = CyclicBarrier::new(
                vec_channel_sender.clone(),
                vec_channel_receiver.pop().unwrap(),
            );
            move || {
                for j in 0..10 {
                    cbarrier.wait();
                    println!("After barrier thread #{} -> Print {}", i, j);
                }
            }
        }));
    }

    for t in vt {
        t.join().unwrap();
    }
}
