use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub fn bundle
    <T: 'static + Send + Sync>
    (receiver: Receiver<T>, bundle_size: usize)
     -> Receiver<Vec<T>> {

    let (tx, rx) = channel();
    thread::spawn(move|| {
        loop {
            let mut current_bundle = Vec::with_capacity(bundle_size);
            for _ in 0..bundle_size {
                let received = receiver.recv()
                    .expect("Received line for bundling");
                current_bundle.push(received);
            }
            tx.send(current_bundle).expect("Sending bundle");
        }
    });

    rx
}
