use std::sync::mpsc::{channel, Receiver};
use std::thread;

use LogLine;

pub type LogBundle = Vec<LogLine>;

pub struct Bundler {
    pub bundle_size: usize,
    pub receiver: Receiver<LogBundle>,
}

impl Bundler {
    pub fn new(log_line_receiver: Receiver<LogLine>, bundle_size: usize) -> Bundler {
        let (tx, rx) = channel();
        thread::spawn(move|| {
            let mut current_bundle: LogBundle = Vec::new();
            loop {
                let log_line = log_line_receiver.recv().unwrap();
                current_bundle.push(log_line);

                if current_bundle.len() == bundle_size {
                    tx.send(current_bundle.clone()).unwrap();
                    current_bundle.clear();
                }
            }
        });

        Bundler {
            bundle_size: bundle_size,
            receiver: rx,
        }
    }
}
