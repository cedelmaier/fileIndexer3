#![feature(plugin)]
#![plugin(regex_macros)]
extern crate regex;

use std::ascii::AsciiExt;
use std::thread;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::io::stdin;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;

pub fn ssfi() {
    let re = regex!(r"\W");
    let word_map: BTreeMap<String, usize> = BTreeMap::new();
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

        for line in BufReader::new(file).lines() {
            let s = line.unwrap();
            send.send(s).unwrap();
        }
    });

    // Run single threaded listeners for now
    let data2 = data.clone();
    let recv_guard = thread::spawn(move || {
        while let Ok(n) = recv.recv() {
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

    // What does the data look like?
    println!("Data:\n {:?}", data);
}
