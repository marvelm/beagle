use std::sync::mpsc::{channel, Receiver};
use std::thread;

use iso8601;

use LogLine;

#[derive(Debug)]
pub struct HerokuRouterLogLine {
    timestamp: iso8601::DateTime,
    at: String,
    method: String,
    path: String,
    host: String,
    request_id: String,
    fwd: String,
    dyno: String,
    connect: i32,
    service: i32,
    status: u8,
    bytes: i32,
}

pub fn parse_router_log_lines(rx: Receiver<LogLine>) -> Receiver<HerokuRouterLogLine> {
    let (tx, heroku_receiver) = channel();
    thread::spawn(move||{
        loop {
            let log_line = rx.recv().unwrap();
            convert_log_line(log_line)
                .map(|heroku_log_line| {
                    tx.send(heroku_log_line).unwrap();
                });
        }
    });
    heroku_receiver
}

pub fn convert_log_line(log_line: LogLine) -> Option<HerokuRouterLogLine> {
    if log_line.logger != "heroku" && log_line.process != "router" {
        return None;
    }

    let parts = log_line.line.split_whitespace();
    let pairs = parts.map(|part| {
        let i = part.chars().position(|char| { char == '=' }).unwrap();
        (part[0..i].to_string(), part[i+1..].to_string())
    });

    let mut at = String::new();
    let mut method = String::new();
    let mut path = String::new();
    let mut host = String::new();
    let mut request_id = String::new();
    let mut fwd = String::new();
    let mut dyno = String::new();
    let mut connect: i32 = 0;
    let mut service: i32 = 0;
    let mut status: u8 = 0;
    let mut bytes: i32 = 0;

    for (key, value) in pairs {
        match key.as_str() {
            "at" => at = value,
            "method" => method = value,
            "path" => path = value.replace("\"", ""),
            "host" => host = value,
            "request_id" => request_id = value,
            "fwd" => fwd = value.replace("\"", ""),
            "dyno" => dyno = value,
            "connect" => connect = parse_ms(value),
            "service" => service = parse_ms(value),
            "status" => status = value.parse().expect("Parsing status code"),
            "bytes" => bytes = value.parse().expect("Parsing bytes"),
            _ => println!("Unknown key {}={}", key, value),
        }
    }

    Some(HerokuRouterLogLine {
        timestamp: log_line.timestamp,
        at: at,
        method: method,
        path: path,
        host: host,
        request_id: request_id,
        fwd: fwd,
        dyno: dyno,
        connect: connect,
        service: service,
        status: status,
        bytes: bytes,
    })
}

fn parse_ms(raw: String) -> i32 {
    let i = raw.find('m').expect("Finding m in XXms");
    raw[0..i].parse().expect("Parsing integer value of ms")
}
