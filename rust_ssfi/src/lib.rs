//extern crate comm;

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel};
use std::{thread};
use std::collections::{HashMap,BTreeMap};
use std::fs::{File};

//use comm::{spmc};

pub fn ssfi() {
    // Start the queue
    //let word_map: HashMap<&str, usize> = HashMap::new();
    let word_map: BTreeMap<&str, usize> = BTreeMap::new();
    let data = Arc::new(Mutex::new(word_map));
    //let nthreads = 2;
    //let (send, recv) = spmc::unbounded::new();
    let (send, recv) = channel::<&str>();
    let input = vec!["The task is",
                     "to write a program which",
                     "accepts",
                     "lines of text and generates output",
                     "lines of",
                     "a different length, without splitting any of the",
                     "words in the text. We assume no word is longer than the size of",
                     "the output lines.",
                     "the quick brown fox jumped over the lazy dog",
                     "she sell sea shell by the sea shore"];

    // Start the sender
    // Use a JoinHandle to explicitly join
    // at the end of the program
    let send_guard = thread::spawn(move || {
        for line in input {
            send.send(line).unwrap();
        }
        // Stick an infinite loop here to
        // Make sure that sender stays open
        // and doesn't close prematurely
    });

    // Run single threaded listeners for now
    // Have to move inside of a scope to clone data
    let data2 = data.clone();
    let recv_guard = thread::spawn(move || {
        while let Ok(n) = recv.recv() {
            let words = n.split(' ');
            for word in words {
                let mut data = data2.lock().unwrap();
                *data.entry(word).or_insert(0) += 1
            }
        }
    });


    // Start the listeners
    // Use a JoinHandle to collect the threads
    // Then start listening, which is a blocking
    // operation.
    //let recv_guards: Vec<_> = (0..nthreads).map( |i| {
    //    let (recv, data) = (recv.clone(), data.clone());
    //    thread::spawn(move || {
    //        // Listen unless the sender has disconnected
    //        while let Ok(n) = recv.recv_sync() {
    //            let words = n.split(' ');
    //            for word in words {
    //                let mut data = data.lock().unwrap();
    //                // Add the data to the map
    //                //let counter = data.entry(word).or_insert(0);
    //                //*counter += 1;
    //                // BTreeMap in place manipulation
    //                *data.entry(word).or_insert(0) += 1;
    //            }
    //        }
    //    })
    //}).collect();

    // Join the sender, then receivers
    //println!("Sender: {:?}", send_guard.join().unwrap());
    //for i in recv_guards {
    //    println!("Receiver: {:?}", i.join().unwrap()); 
    //}
    println!("Sender: {:?}", send_guard.join().unwrap());
    println!("Receiver: {:?}", recv_guard.join().unwrap());

    // What does the data look like?
    println!("Data:\n {:?}", data);
}

