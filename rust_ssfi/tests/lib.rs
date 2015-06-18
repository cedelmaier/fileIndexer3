extern crate comm;

use std::thread;
use comm::{spsc,mpsc};

#[test]
fn test_spsc() {
    // Create a bounded SPSC channel.
    let (send, recv) = spsc::bounded::new(10);
    thread::spawn(move || {
        send.send_sync(10).unwrap();
    });
    assert_eq!(recv.recv_sync().unwrap(), 10);
}

