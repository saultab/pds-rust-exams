use std::sync::mpsc::{channel, Sender};
use std::thread;

pub struct Looper<M : 'static> {
    tx: Sender<M>,
}

impl<M : 'static> Looper<M> {
    pub fn new(process: impl Fn(M) -> () + Send + 'static, cleanup: impl FnOnce() -> () + Send + 'static) -> Self {
        let (tx, rx) = channel::<M>();
        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                process(msg);
            }
            cleanup();
        });

        Looper {
            tx
        }
    }

    fn send<M>(&self, msg: M) {
        self.tx.send(msg).unwrap();
    }
}