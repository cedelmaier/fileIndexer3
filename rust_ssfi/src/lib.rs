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

    // Start the sender
    // Use a JoinHandle to explicitly join
    // at the end of the program
    let send_guard = thread::spawn(move || {
        // Read in a file and send it
        let path = Path::new("../../test/Clone0/Clone1/completeWorks.txt");
        let file = match File::open(&path) {
            Err(why) => panic!("failed to open {}: {}", path.display(), why),
            Ok(f) => f,
        };

        // Read the lines from the file and send them
        for line in BufReader::new(file).lines() {
            let s = line.unwrap();
            send.send(s).unwrap();
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
                // Convert to lowercase, then split on anything
                // that isn't alphanumeric
                let ln = n.to_ascii_lowercase();
                // The regex! must be compiled here, not in the main program
                // or we have to borrow it or something.  Is it faster than
                // the dynamic regex version?
                let re = regex!(r"\W");
                let words: Vec<&str> = re.split(&ln).collect();
                for word in words {
                    match word {
                        "" => continue,
                        _ => {
                            // Lock the data and add to it, this should be the
                            // bottleneck in the code
                            let mut data = data.lock().unwrap();
                            *data.entry(word.to_string()).or_insert(0) += 1;
                        },
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

