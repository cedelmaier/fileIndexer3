#![feature(test)]

extern crate comm;

#[cfg(test)]
mod tests {
    //use super::*;
    extern crate test;

    use std::thread;
    use std::sync::{Arc, Mutex};

    use comm::{spsc, spmc};

    #[test]
    fn test_spsc() {
        // Create a bounded SPSC channel.
        let (send, recv) = spsc::bounded::new(10);
        thread::spawn(move || {
            send.send_sync(10).unwrap();
        });
        assert_eq!(recv.recv_sync().unwrap(), 10);
    }

    #[test]
    fn test_spmc() {
        // Created an unbounded SPMC channel
        // Complicated integration test, mirrors
        // functionality in main program
        let results = Arc::new(Mutex::new(Vec::new()));
        let (send, recv) = spmc::unbounded::new();
        let send_guard = thread::spawn(move || {
            for i in 0..10 {
                send.send(i).unwrap();
            }
        });

        let recv_guards: Vec<_> = (0..2).map(|_| {
            let (recv, results) = (recv.clone(), results.clone());
            thread::spawn(move || {
                while let Ok(n) = recv.recv_sync() {
                    let mut result = results.lock().unwrap();
                    result.push(n);
                }
            })
        }).collect();

        send_guard.join().unwrap();
        for i in recv_guards {
            i.join().unwrap();
        }
        let unwrap_results = results.lock().unwrap();
        assert_eq!(unwrap_results.iter().fold(0, |sum, x| sum + x),
                                 (0..10).fold(0, |sum, x| sum + x));
    }
}

