#![feature(plugin)]
#![feature(fs_walk)]
#![plugin(regex_macros)]

extern crate comm;
extern crate regex;

use std::ascii::AsciiExt;
use std::thread;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use std::sync::{Arc, Mutex};

use regex::Regex;

use comm::spmc;

pub fn ssfi(nthreads: usize, directory: &str) {
    // Set up any persistent variables
    let word_map: HashMap<String, usize> = HashMap::new();
    let data = Arc::new(Mutex::new(word_map));
    let (send, recv) = spmc::unbounded::new();

    // Start the sender
    // Use a JoinHandle to explicitly join
    // at the end of the program
    // Need a String, &str doesn't live long enough
    // Can we fix this?  Happens a lot
    let s_directory: String = directory.to_string();
    let send_guard = thread::spawn(move || {
        match fs::walk_dir(s_directory) {
            Err(why) => println!("! {:?}", why),
            Ok(paths) => for path in paths {
                // Incredibly arcane conversion from a 
                // DirEntry to a String
                let s: String = path.unwrap()
                                    .path()
                                    .to_str()
                                    .unwrap()
                                    .to_string();
                // Try using the normal regexes, test for
                // .txt files ending the filename
                let re = Regex::new(r"\.txt$").unwrap();
                if re.is_match(&s) {
                    send.send(s).unwrap();
                }
            },
        }
    });


    // Start the listeners
    // Use a JoinHandle to collect the threads
    // Then start listening, which is a blocking
    // operation.
    let recv_guards: Vec<_> = (0..nthreads).map( |i| {
        let (recv, data) = (recv.clone(), data.clone());
        thread::spawn(move || {
            println!("Indexer[{}] coming online", i);
            // Listen unless the sender has disconnected
            while let Ok(n) = recv.recv_sync() {
                // Create the path and attempt to open
                println!("\tIndexer[{}] indexing: {}", i, n);
                // Path::new takes a string slice
                let path = Path::new(&n);
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
                    // version?  Can we just borrow it for the two closures?
                    //let re = regex!(r"\W");
                    let re = Regex::new(r"\W").unwrap();
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

            println!("Indexer[{}] terminating", i);
        })
    }).collect();

    // Join the sender, then receivers
    println!("Sender: {:?}", send_guard.join().unwrap());
    for i in recv_guards {
        println!("Receiver: {:?}", i.join().unwrap()); 
    }

    // Print the first 20 keys in the map for fun
    // Unordered since we're using a hashmap
    let mut counter = 0;
    for (key, value) in data.lock().unwrap().iter() {
        if counter > 20 { break; }
        println!("{}: {}", key, value);
        counter += 1;
    }
    // What does the data look like?
    //println!("Data:\n {:?}", data);
}

