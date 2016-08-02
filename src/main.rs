#[macro_use] extern crate hyper;
#[macro_use] extern crate nom;
extern crate iso8601;
extern crate regex;

use std::env;
use std::io::{BufRead, BufReader, Read};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use hyper::client::Client;
use hyper::header::{Authorization, Basic};

mod parser;

fn main() {
    let mut args = env::args();
    if args.nth(1).expect("client") == "client" {
        println!("client mode");

        for log_line in client_mode() {
            println!("{:?}", log_line);
        }
    }
}

pub fn client_mode() -> Receiver<LogLine>{
    let (tx, rx) = channel();

    thread::spawn(move|| {
        'start: loop {
            let tail_url = get_tail_url();
            let mut client = Client::new();
            client.set_read_timeout(None);

            let response = client.get(&tail_url).send()
                .expect("Unable to connect to Heroku");
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

fn parse_log_line(line: std::io::Result<String>) -> Option<LogLine> {
    match line {
        Ok(s) => parser::parse_log_line(&s),
        Err(e) => {
            println!("{}, {}", "Error reading line", e);
            None
        }
    }
}

fn get_tail_url() -> String {
    let username = env::var("username").expect("username env var");
    let password = env::var("password").expect("password env var");
    let app_name = env::var("app_name").expect("app_name env var");

    let client = Client::new();
    let url = format!("https://api.heroku.com/apps/{}/logs?logplex=true&tail=1",
                      app_name);
    let mut response = client.get(&url)
        .header(Authorization(Basic {
            username: username,
            password: Some(password)
        }))
        .send()
        .expect("Connection error while getting tail url");

    let mut tail_url = String::new();
    response.read_to_string(&mut tail_url)
        .expect("Unable to read tail url");
    return tail_url;
}

#[derive(Debug, Clone)]
pub struct LogLine {
    timestamp: iso8601::DateTime,
    logger: String,
    process: String,
    line: String,
}
