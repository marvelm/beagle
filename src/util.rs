use std::fs::File;
use std::env;
use std::io::{Error, ErrorKind, Read, Result};

use hyper::client::Client;
use hyper::header::{Authorization, Basic};
use regex::Regex;

pub fn get_netrc_creds() -> Result<(String, String)> {
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
        Err(Error::new(ErrorKind::NotFound,
                       "Try logging into Heroku first"))
    } else {
        Ok((String::from(username),
            String::from(password)))
    }
}

pub fn get_tail_url() -> Result<String> {
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
