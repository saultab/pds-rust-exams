use std::sync::mpsc::{channel, Receiver, Sender};

static NUMBER_OF_THREAD: usize = 3;

struct CyclicBarrierMaster {
    sender: Vec<Sender<bool>>,
    receiver: Receiver<bool>,
}

struct CyclicBarrierSlave {
    sender: Sender<bool>,
    receiver: Receiver<bool>,
}

impl CyclicBarrierMaster {
    pub fn new(sender: Vec<Sender<bool>>, receiver: Receiver<bool>) -> Self {
        CyclicBarrierMaster {
            sender,
            receiver,
        }
    }
    pub fn wait_master(&mut self) {

        //Wait the PASS token from master
        for _ in 0..NUMBER_OF_THREAD {
            self.receiver.recv().unwrap();
        }

        //Forward the PASS tokens to slave
        for i in 0..NUMBER_OF_THREAD {
            self.sender[i].send(true).expect("Error forward token");
        }
    }
}

impl CyclicBarrierSlave {
    pub fn new(sender: Sender<bool>, receiver: Receiver<bool>) -> Self {
        CyclicBarrierSlave {
            sender,
            receiver,
        }
    }

    pub fn wait_slave(&mut self) {

        //Forward token to master
        self.sender.send(true).expect("Error forward token");

        //Wait the PASS token from master
        self.receiver.recv().unwrap();
    }
}

fn main() {
    let mut vec_channel_receiver = Vec::<Receiver<bool>>::new();
    let mut vec_channel_sender = Vec::<Sender<bool>>::new();

    // Create a vec of channels for link MASTER->SLAVE
    for _ in 0..NUMBER_OF_THREAD {
        let (sender, receiver) = channel();
        vec_channel_sender.push(sender);
        vec_channel_receiver.push(receiver);
    }

    let (sender_master, receiver_master) = channel();

    let mut vt = Vec::new();

    //Spawn the Master
    vt.push(std::thread::spawn({
        let mut cbarrier = CyclicBarrierMaster::new(
            vec_channel_sender.clone(),
            receiver_master,
        );
        move || {
            for _ in 0..10 {
                cbarrier.wait_master();
            }
        }
    }));

    //Cycle from #threads to zero because pop is the only method that takes ownership
    for i in (0..NUMBER_OF_THREAD).rev() {
        vt.push(std::thread::spawn({
            let mut cbarrier = CyclicBarrierSlave::new(
                sender_master.clone(),
                vec_channel_receiver.pop().unwrap(),
            );
            move || {
                for j in 0..10 {
                    cbarrier.wait_slave();
                    println!("After barrier thread #{} -> Print {}", i, j);
                }
            }
        }));
    }

    for t in vt {
        t.join().unwrap();
    }
}
