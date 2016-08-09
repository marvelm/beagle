use iso8601::parsers::parse_datetime;
use nom::{rest, space};

use LogLine;
use std::str::from_utf8;

fn bytes_to_string(bytes: &[u8]) -> String {
    String::from(from_utf8(bytes)
                 .expect("Converting bytes to string"))
}

named!(log_line_parser( &[u8] ) -> LogLine, chain!(
    timestamp: parse_datetime ~
        space ~ // Space
        logger_name: take_until!(b"[") ~
        take!(1) ~ // Consume '['
        process_name: take_until!(b"]") ~
        take!(2) ~ // Consume ']' and ':'
        space ~
        line: rest
        ,
        || {
            LogLine {
                timestamp: timestamp,
                logger: bytes_to_string(logger_name),
                process: bytes_to_string(process_name),
                line: bytes_to_string(line),
            }
        }));

use nom::IResult;

pub fn parse_log_line(raw_line: &str) -> Option<LogLine> {
    let bytes = raw_line.as_bytes();
    let res = log_line_parser(bytes);

    match res {
        IResult::Done(_, out) => Some(out),
        IResult::Incomplete(needed) => {
            println!("{:?}", needed);
            None
        },
        _ => None,
    }
}
