extern crate beagle;

use std::env;

use beagle::{LogLine, client_mode};
use beagle::pipes::heroku::{HerokuRouterLogLine, parse_router_log_lines};
use beagle::pipes::bundle;
use std::sync::mpsc::Receiver;

fn main() {
    let mut args = env::args();
    if args.nth(1).expect("client") == "client" {
        println!("client mode");

        let log_line_rx: Receiver<LogLine> = client_mode();

        let bundle_size = 12;
        let rx = bundle(parse_router_log_lines(log_line_rx), bundle_size);
        loop {
            let log_bundle = rx.recv().unwrap();
            analyze_bundle(log_bundle)
        }
    }
}

fn analyze_bundle(log_bundle: Vec<HerokuRouterLogLine>) {
    let mut num_500 = 0;
    for line in log_bundle {
        if line.status == 500 {
            num_500 += 1;
        }
    }
    if num_500 > 2 {
        println!("Something's going on");
    }
}
