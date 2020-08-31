// MIT/Apache2 License

#[cfg(feature = "pl")]
use std::{time::Duration, thread};

// spawns a quick deadlock detector
#[cfg(feature = "pl")]
pub fn deadlock_detector() {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(10));

            let deadlocks = parking_lot::deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{}", i);
                for t in threads {
                    println!("Thread Id {:#?}", t.thread_id());
                    println!("{:#?}", t.backtrace());
                }
            }
        }
    });
}

#[cfg(not(feature = "pl"))]
pub fn deadlock_detector() {}
