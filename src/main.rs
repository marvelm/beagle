#[macro_use] extern crate hyper;
#[macro_use] extern crate nom;
extern crate iso8601;
extern crate regex;

use std::env;
use std::io::{BufRead, BufReader, Read, Result};
use std::fs::File;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use hyper::client::Client;
use hyper::header::{Authorization, Basic};
use regex::Regex;

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
            let tail_url = get_tail_url()
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

fn parse_log_line(line: std::io::Result<String>) -> Option<LogLine> {
    match line {
        Ok(s) => parser::parse_log_line(&s),
        Err(e) => {
            println!("{}, {}", "Error reading line", e);
            None
        }
    }
}

fn get_netrc_creds() -> std::io::Result<(String, String)> {
    let mut location = env::home_dir()
        .expect("Getting home directory");
    location.push(".netrc");

    let mut file = try!(File::open(location));
    let mut file_contents = String::new();
    try!(file.read_to_string(&mut file_contents));

    let lines = file_contents.lines();

    let mut not_found_heroku = true;
    let mut username = "";
    let mut password = "";

    for line in lines {
        if not_found_heroku {
            if line.contains("heroku.com") {
                not_found_heroku = false;
            }
        } else if username == "" {
            let re = Regex::new("login (.+)").unwrap();
            username = re.captures(line)
                .expect("Searching for login")
                .at(1).expect("Username group");
        } else if password == "" {
            let re = Regex::new("password (.+)").unwrap();
            password = re.captures(line)
                .expect("Searching for password")
                .at(1).expect("Password group");
            break;
        }
    }

    if username == "" || password == "" {
        Err(std::io::Error::new(std::io::ErrorKind::NotFound,
                            "Try logging into Heroku first"))
    } else {
        Ok((String::from(username),
            String::from(password)))
    }
}

fn get_tail_url() -> Result<String> {
    let (username, password) = try!(get_netrc_creds());
    let app_name = env::var("app_name")
        .expect("app_name env var");

    let client = Client::new();
    let url = format!("https://api.heroku.com/apps/{}/logs?logplex=true&tail=1",
                      app_name);
    let mut response = client
        .get(&url)
        .header(Authorization(Basic {
            username: username,
            password: Some(password)
        }))
        .send()
        .expect("Reading tail url response");

    let mut tail_url = String::new();
    try!(response.read_to_string(&mut tail_url));
    return Ok(tail_url);
}

#[derive(Debug, Clone)]
pub struct LogLine {
    timestamp: iso8601::DateTime,
    logger: String,
    process: String,
    line: String,
}
