#![feature(plugin)]
#![feature(fs_walk)]
#![plugin(regex_macros)]
#![feature(test)]

extern crate comm;
extern crate regex;
extern crate concurrent_hashmap;

use std::thread;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;
use std::path::Path;
use std::sync::Arc;

use regex::Regex;

use comm::spmc;

use concurrent_hashmap::*;

pub fn ssfi(nthreads: usize, directory: &str, printon: bool) {
    // Set up any persistent variables
    let mut data_opt: Options<_> = Default::default();
    data_opt.capacity = 524_288; // Roughly half the words in english
    data_opt.concurrency = 64; // Finer grained concurrency
    let data: Arc<ConcHashMap<String, usize>> = Arc::new(ConcHashMap::with_options(data_opt));
    let (send, recv) = spmc::unbounded::new();

    // Start the sender
    // Use a JoinHandle to explicitly join
    // at the end of the program
    // Need a String, &str doesn't live long enough
    // Can we fix this?  Happens a lot
    let directory: String = directory.to_string();
    let send_guard = thread::spawn(move || {
        match fs::walk_dir(directory) {
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
                    let ln: String = line.unwrap();
                    let words: Vec<String> = ln.split(|w: char| !w.is_alphanumeric())
                                               .map(|w| w.to_lowercase())
                                               .filter(|w| !w.is_empty())
                                               .collect();
                    for word in words {
                        // Insert into the concurrent hashmap
                        data.upsert(word.to_owned(), 1, &|count| *count += 1);
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

    if printon{
        let mut counts: Vec<(String, usize)> = data.iter().map(|(s, &n)| (s.clone(), n)).collect();
        counts.sort_by(|&(_, a), &(_, b)| b.cmp(&a));
        for (i, &(ref word, count)) in counts.iter().enumerate() {
            if i >= 10 { break; }
            println!("[{}]\t{}", word, count);
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;

    #[bench]
    fn ssfi_test_t1(b: &mut test::Bencher) {
        b.iter(|| test::black_box(ssfi(1, "../../test/Clone0/Clone1/", false)))
    } 

    #[bench]
    fn ssfi_test_t2(b: &mut test::Bencher) {
        b.iter(|| test::black_box(ssfi(2, "../../test/Clone0/Clone1/", false)))
    } 
}
