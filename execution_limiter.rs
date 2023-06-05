use std::fmt::Display;
use std::sync::{Arc, Condvar, Mutex};
use std::thread::sleep;
use std::time::Duration;

static N: usize = 3;

struct ExecutionLimiter {
    limit_value: Mutex<usize>,
    cv: Condvar,
}

impl ExecutionLimiter {
    fn new() -> Self {
        return ExecutionLimiter { limit_value: Mutex::new(0), cv: Default::default() };
    }

    fn execute<R : Display>(&self, func: impl FnOnce(R) -> R, param: R) -> R {
        let mut limit = self.limit_value.lock().unwrap();

        //metodo 0 per attendere cv
        //limit = cvar.wait_while(limit, *limit == N).unwrap();

        //metodo 1 per attendere cv
        while *limit == N-1 {
            println!("> In attesa...");
            limit = self.cv.wait(limit).unwrap();
        }

        // altrimenti
        // *(self.limitValue.lock().unwrap()) += 1;

        *limit += 1;
        println!("> Limite incrementato a {}", *limit);
        drop(limit);

        let ret = func(param);

        let mut limit = self.limit_value.lock().unwrap();
        *limit -= 1;
        println!("> Limite de-crementato a {}", *limit);
        drop(limit);

        self.cv.notify_one();
        println!("> Fatto!\n");
        ret
    }
}

pub fn f<R : Display>(i: R) -> R {
    println!("----------> print {i} funzione f");
    sleep(Duration::from_secs(5));
    i
}

fn main() {
    let limiter = Arc::new( ExecutionLimiter::new());
    let mut vt = Vec::new();

    for i in 0..5 {
        vt.push(std::thread::spawn({
            let l = limiter.clone();
            move || {
                loop {
                    println!("> thread #{i} entra");
                    l.execute(f, i);
                    println!("> thread #{i} esce");
                    sleep(Duration::from_secs(10));
                }
            }
        }
        ));
    }

    for t in vt {
        t.join().unwrap();
    }
}