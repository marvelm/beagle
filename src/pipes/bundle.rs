use std::sync::mpsc::{channel, Receiver};
use std::thread;

pub fn bundle
    <T: 'static + Send + Sync + Clone>
    (receiver: Receiver<T>, bundle_size: usize)
     -> Receiver<Vec<T>> {

    let (tx, rx) = channel();
    thread::spawn(move|| {
        let mut current_bundle = Vec::new();
        loop {
            let received = receiver.recv().unwrap();
            current_bundle.push(received);

            if current_bundle.len() == bundle_size {
                tx.send(current_bundle.clone()).unwrap();
                current_bundle.clear();
            }
        }
    });

    rx
}
