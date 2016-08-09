extern crate beagle;
extern crate rustty;

mod render;

use std::env;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use rustty::{Terminal, Event};

use beagle::{LogLine, client_mode};
use beagle::pipes::heroku::{HerokuRouterLogLine, parse_router_log_lines};
use beagle::pipes::bundle;

fn main() {
    let mut args = env::args();
    if args.nth(1) .expect("Reading first arg") == "client" {
        let mut term = Terminal::new().unwrap();
        term.swap_buffers().unwrap();

        let log_lines: Receiver<LogLine> = client_mode();
        let router_lines: Receiver<HerokuRouterLogLine> = parse_router_log_lines(log_lines);

        let bundle_size = 12;
        render::error_rate(&mut term, 0, bundle_size);

        let bundles = bundle(router_lines, bundle_size);

        let timeout = Duration::from_millis(100);
        loop {
            if let Some(Event::Key(ch)) = term.get_event(timeout).unwrap() {
                match ch {
                    '`' => break,
                    _ => continue,
                }
            }

            let log_bundle = bundles.recv().unwrap();
            let num_errors = log_bundle.iter().fold(0, |acc, log_line| {
                if log_line.status == 500 {
                    acc + 1
                } else {
                    acc
                }
            });

            render::error_rate(&mut term, num_errors, bundle_size);
            render::sample_line(&mut term,
                                log_bundle.first().unwrap());
            term.swap_buffers().unwrap();
        }
    }
}
