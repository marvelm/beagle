extern crate beagle;
extern crate rustty;

use std::env;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;

use rustty::{Color, Terminal, Event, HasSize};
use rustty::ui::{Painter, Dialog, DialogResult, Alignable, HorizontalAlign, VerticalAlign, Widget};

use beagle::{LogLine, client_mode};
use beagle::pipes::heroku::{HerokuRouterLogLine, parse_router_log_lines};
use beagle::pipes::bundle;

fn render_error_rate(term: &mut Terminal,
                     num_errors: usize, bundle_size: usize) {
    let mut dialog = Dialog::new(50, 6);
    let mut msg = String::new();
    msg.push_str(&num_errors.to_string());
    msg.push('/');
    msg.push_str(&bundle_size.to_string());
    msg.push_str(" requests have been status_code=500");

    dialog.window_mut().align(term,
                              HorizontalAlign::Left,
                              VerticalAlign::Top, 1);
    dialog.window_mut().printline(1, 1, &msg);
    dialog.window_mut().draw_box();
    dialog.window().draw_into(term);
}

fn render_sample_line(term: &mut Terminal,
                      line: &str) {
    let mut dialog = Dialog::new(100, 6);
    let mut msg = String::new();
    msg.push_str("Sample request: ");
    msg.push_str(line);

    dialog.window_mut().align(term,
                          HorizontalAlign::Left,
                          VerticalAlign::Bottom, 1);
    dialog.window_mut().printline(1, 1, &msg);
    dialog.window().draw_into(term);
}

fn main() {
    let mut args = env::args();
    if args.nth(1).expect("client") == "client" {
        let mut term = Terminal::new().unwrap();
        term.swap_buffers().unwrap();

        let log_lines: Receiver<LogLine> = client_mode();
        let router_lines: Receiver<HerokuRouterLogLine> = parse_router_log_lines(log_lines);
        let bundle_size = 12;
        render_error_rate(&mut term, 0, bundle_size);
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

            render_error_rate(&mut term, num_errors, bundle_size);
            render_sample_line(&mut term,
                               &log_bundle.first().unwrap().path);
            term.swap_buffers().unwrap();
        }
    }
}
