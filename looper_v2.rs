use std::sync::mpsc::{channel, Sender};
use std::thread;

pub struct Looper<M: Send + 'static> {
    tx: Sender<M>,
}

impl <M: Send +'static> Looper<M> {
    pub fn new(process: impl Fn(M) -> () + Send + 'static, cleanup: impl FnOnce() -> () + Send + 'static) -> Self {
        let (tx, rx) = channel::<M>();
        thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                process(msg);
            }
            cleanup();
        });

        return Looper { tx }
    }

    fn send(&self, msg: M) {
        self.tx.send(msg).unwrap();
    }
}