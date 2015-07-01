#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

use std::ascii::AsciiExt;
use std::thread;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

pub fn ssfi() {
    // Set up any persistent? variables
    let re = regex!(r"\W");
    let word_map: HashMap<String, usize> = HashMap::new();
    let data = Arc::new(Mutex::new(word_map));
    let (send, recv) = channel::<String>();

    // Start the sender
    // Use a JoinHandle to explicitly join
    // at the end of the program
    let send_guard = thread::spawn(move || {
        // Read in a file and send it
        let path = Path::new("hamlet.txt");
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

    // Run single threaded listeners for now
    // Create a data2 clone from data for the receivers
    let data2 = data.clone();
    let recv_guard = thread::spawn(move || {
        while let Ok(n) = recv.recv() {
            // Convert to lowercase, then split on anything that
            // isn't alphanumeric
            let ln = n.to_ascii_lowercase();
            let words: Vec<&str> = re.split(&ln).collect();
            for word in words {
                match word {
                    "" => continue,
                    _ => {
                        let mut data = data2.lock().unwrap();
                        *data.entry(word.to_string()).or_insert(0) += 1;
                    },
                }
            }
        }
    });

    println!("Sender: {:?}", send_guard.join().unwrap());
    println!("Receiver: {:?}", recv_guard.join().unwrap());

    // Print the first 20 key value pairs in the tree
    let mut counter = 0;
    for (key, value) in data.lock().unwrap().iter() {
        if counter > 20 { break; }
        println!("{}: {}", key, value);
        counter += 1;
    }
    // The whole dataset
    // println!("{:?}", data);
}

