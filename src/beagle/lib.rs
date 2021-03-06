#[macro_use] extern crate hyper;
#[macro_use] extern crate nom;
extern crate iso8601;
extern crate regex;

pub mod pipes;
mod util;
mod parser;

use std::io::{BufRead, BufReader, Result};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use hyper::client::Client;

#[derive(Debug, Clone)]
pub struct LogLine {
    timestamp: iso8601::DateTime,
    logger: String,
    process: String,
    line: String,
}

pub fn client_mode() -> Receiver<LogLine>{
    let (tx, rx) = channel();

    thread::spawn(move|| {
        'start: loop {
            let tail_url = util::get_tail_url()
                .expect("Getting tail url");
            let mut client = Client::new();
            client.set_read_timeout(None);

            let response = client.get(&tail_url).send()
                .expect("Connecting to Heroku");
            let buf = BufReader::new(response);

            for line in buf.lines() {
                match parse_log_line(line) {
                    Some(log_line) => tx.send(log_line)
                        .expect("Unable to send log_line"),
                    None => continue 'start
                }
            }
        }
    });

    rx
}

fn parse_log_line(line: Result<String>) -> Option<LogLine> {
    match line {
        Ok(s) => parser::parse_log_line(&s),
        Err(e) => {
            println!("{}, {}", "Error reading line", e);
            None
        }
    }
}
