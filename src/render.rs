use rustty::Terminal;
use rustty::ui::{Painter, Dialog, Alignable, HorizontalAlign, VerticalAlign};

use beagle::pipes::heroku::{HerokuRouterLogLine};

pub fn error_rate(term: &mut Terminal,
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

pub fn sample_line(term: &mut Terminal,
               line: &HerokuRouterLogLine) {
    let mut dialog = Dialog::new(100, 6);
    dialog.window_mut().align(term,
                              HorizontalAlign::Left,
                              VerticalAlign::Bottom, 1);
    dialog.window_mut().printline(0, 1, &line.path);
    dialog.window_mut().printline(0, 2, &line.request_id);
    dialog.window().draw_into(term);
}
