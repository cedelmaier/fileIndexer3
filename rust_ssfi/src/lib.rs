#![feature(plugin)]
#![feature(fs_walk)]
#![plugin(regex_macros)]
#![feature(test)]

extern crate comm;
extern crate regex;

use std::thread;
use std::ascii::AsciiExt;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use std::sync::{Arc, Mutex};

use regex::Regex;

use comm::spmc;

pub fn ssfi(nthreads: usize, directory: &str, printon: bool) {
    // Set up any persistent variables
    let data = Arc::new(Mutex::new(HashMap::<String, u32>::new()));
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

    let re = regex!(r"[^a-zA-Z0-9_]+"); // Fastest
    //let re = regex!(r"[a-zA-z0-9_]+");

    // Start the listeners
    // Use a JoinHandle to collect the threads
    // Then start listening, which is a blocking
    // operation.
    let recv_guards: Vec<_> = (0..nthreads).map( |i| {
        let (recv, data, re) = (recv.clone(), data.clone(), re.clone());
        thread::spawn(move || {
            if printon { println!("Indexer[{}] coming online", i); }
            // Listen unless the sender has disconnected
            while let Ok(n) = recv.recv_sync() {
                // Create the path and attempt to open
                if printon { println!("\tIndexer[{}] indexing: {}", i, n); }
                let path = Path::new(&n);
                let file = match File::open(&path) {
                    Err(why) => panic!("failed to open {}: {}", path.display(), why),
                    Ok(f) => f,
                };

                // Now read the lines from the file
                for line in BufReader::new(file).lines() {
                    let ln = line.unwrap().to_ascii_lowercase();
                    // regex.split is a huge bottleneck!!!!
                    let words: Vec<&str> = re.split(&ln).collect();
                    for word in words {
                        match word {
                            "" => continue,
                            _ => {
                                // Lock the data and add to it, this should be
                                // the bottleneck in the code
                                let mut data = data.lock().unwrap();
                                *data.entry(word.to_owned()).or_insert(0) += 1;
                            },
                        }
                    }
                } // BufReader
            }

            if printon { println!("Indexer[{}] terminating", i); }
        })
    }).collect();

    // Join the sender, then receivers
    send_guard.join().unwrap();
    for i in recv_guards {
        i.join().unwrap();
    }

    // Prints alphabetically
    let mut counter = 0;
    if printon {
        let data = data.lock().unwrap();
        let mut words: Vec<&String> = data.keys().collect();
        words.sort();
        for &word in &words {
            if counter >= 20 { break; }
            if let Some(count) = data.get(word) {
                println!("[{}]\t{}", count, word);
            }
            counter += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;

    #[bench]
    fn ssfi_test_t1(b: &mut test::Bencher) {
        b.iter(|| test::black_box(ssfi(1, "../../test/Clone0/Clone1/books", false)))
    } 

    #[bench]
    fn ssfi_test_t2(b: &mut test::Bencher) {
        b.iter(|| test::black_box(ssfi(2, "../../test/Clone0/Clone1/books", false)))
    } 
}
