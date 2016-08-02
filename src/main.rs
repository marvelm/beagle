#[macro_use] extern crate hyper;
#[macro_use] extern crate nom;
extern crate regex;
extern crate iso8601;

use std::env;
use std::io::{BufRead, BufReader, Read};

use hyper::client::Client;
use hyper::header::{Authorization, Basic};

mod parser;

fn main() {
    let mut args = env::args();
    if args.nth(1).expect("client") == "client" {
        println!("client mode");
        client_mode(10);
    }
}

fn client_mode(frame_size: usize) {
    'start: loop {
        let tail_url = get_tail_url();
        let mut client = Client::new();
        client.set_read_timeout(None);

        let response = client.get(&tail_url).send().unwrap();
        let buf = BufReader::new(response);

        let mut current_frame: Vec<LogLine> = vec![];

        for line in buf.lines() {
            if current_frame.len() == frame_size {
                analyze_log_frame(current_frame.clone());
                current_frame.clear();
            }

            match line {
                Ok(s) => {
                    let log_line = parser::parse_log_line(&s);
                    current_frame.push(log_line.unwrap());
                },
                _ => {
                    continue 'start;
                },
            }
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
        .send().unwrap();

    let mut tail_url = String::new();
    response.read_to_string(&mut tail_url).unwrap();
    return tail_url;
}

#[derive(Debug, Clone)]
pub struct LogLine {
    timestamp: iso8601::DateTime,
    logger: String,
    process: String,
    line: String,
}

fn analyze_log_frame(log_frame: Vec<LogLine>) {
    for line in log_frame {
        println!("{:?}", line);
    }
}
