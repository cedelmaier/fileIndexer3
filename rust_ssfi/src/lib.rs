#![feature(plugin)]
#![plugin(regex_macros)]
extern crate comm;
extern crate regex;

use std::ascii::AsciiExt;
use std::thread;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use std::sync::{Arc, Mutex};

use comm::spmc;

pub fn ssfi() {
    // Set up any persistent variables
    let word_map: HashMap<String, usize> = HashMap::new();
    let data = Arc::new(Mutex::new(word_map));
    let nthreads = 2;
    let (send, recv) = spmc::unbounded::new();

    let input_paths = vec!["../../test/Clone0/Clone1/completeWorks.txt",
                           "../../test/Clone0/Clone1/Oxford English Dictionary.txt",
                           "../../test/Clone0/Clone1/ThreeStateDI.txt",
                           "../../test/Clone0/Clone1/semaphore.txt",
                           "../../test/Clone0/Clone1/books/theadventuresofsherlockholmes.txt"];

    // Start the sender
    // Use a JoinHandle to explicitly join
    // at the end of the program
    let send_guard = thread::spawn(move || {
        // Simply send a file from the input_paths list to the waiting
        // workers
        for path in input_paths {
            send.send(path).unwrap();
        }
    });


    // Start the listeners
    // Use a JoinHandle to collect the threads
    // Then start listening, which is a blocking
    // operation.
    let recv_guards: Vec<_> = (0..nthreads).map( |_| {
        let (recv, data) = (recv.clone(), data.clone());
        thread::spawn(move || {
            // Listen unless the sender has disconnected
            while let Ok(n) = recv.recv_sync() {
                // Create the path and attempt to open
                let path = Path::new(n);
                let file = match File::open(&path) {
                    Err(why) => panic!("failed to open {}: {}", path.display(), why),
                    Ok(f) => f,
                };

                // Now read the lines from the file
                for line in BufReader::new(file).lines() {
                    // Convert every line to lowercase
                    let ln = line.unwrap().to_ascii_lowercase();
                    // The regex! macro must be invoked here otherwise it
                    // won't borrow correctly. Is it faster than the dynamic
                    // version?
                    let re = regex!(r"\W");
                    let words: Vec<&str> = re.split(&ln).collect();
                    for word in words {
                        match word {
                            "" => continue,
                            _ => {
                                // Lock the data and add to it, this should be
                                // the bottleneck in the code
                                let mut data = data.lock().unwrap();
                                *data.entry(word.to_string()).or_insert(0) += 1;
                            },
                        }
                    }
                }
            }
        })
    }).collect();

    // Join the sender, then receivers
    println!("Sender: {:?}", send_guard.join().unwrap());
    for i in recv_guards {
        println!("Receiver: {:?}", i.join().unwrap()); 
    }

    // Print the first 20 keys in the map for fun
    let mut counter = 0;
    for (key, value) in data.lock().unwrap().iter() {
        if counter > 20 { break; }
        println!("{}: {}", key, value);
        counter += 1;
    }
    // What does the data look like?
    //println!("Data:\n {:?}", data);
}

