#[macro_use] extern crate hyper;
extern crate chrono;
extern crate regex;

use std::env;
use std::io::{BufRead, BufReader, Read};

use chrono::{DateTime, FixedOffset};
use hyper::client::Client;
use hyper::header::{Authorization, Basic};

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
                    let log_line = parse_log_line(s);
                    current_frame.push(log_line);
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
struct LogLine {
    timestamp: DateTime<FixedOffset>,
    logger: String,
    process: String,
    line: String,
}

fn parse_log_line(raw_line: String) -> LogLine {
    let mut pieces = raw_line.split_whitespace();
    let raw_timestamp = pieces.next().unwrap();
    let logger = pieces.next().unwrap();

    let mut remaining = String::new();
    for piece in pieces {
        remaining.push_str(" ");
        remaining.push_str(piece);
    }

    LogLine {
        timestamp: DateTime::parse_from_rfc3339(raw_timestamp).unwrap(),
        logger: logger.to_string(),
        process: logger.to_string(),
        line: remaining,
    }
}

fn analyze_log_frame(log_frame: Vec<LogLine>) {
    for line in log_frame {
        println!("{:?}", line);
    }
}
