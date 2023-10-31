use backoff::{Error, ExponentialBackoff};
use reqwest::Url;

use std::fmt::Display;
use std::io::{self, Read};

fn new_io_err<E: Display>(err: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err.to_string())
}

fn fetch_url(url: &str) -> Result<String, Error<io::Error>> {
    let op = || {
        println!("Fetching {}", url);
        let url = Url::parse(url)
            .map_err(new_io_err)
            // Permanent errors need to be explicitly constructed.
            .map_err(Error::Permanent)?;

        let mut resp = reqwest::blocking::get(url)
            // Transient errors can be constructed with the ? operator
            // or with the try! macro. No explicit conversion needed
            // from E: Error to backoff::Error;
            .map_err(new_io_err)?;

        let mut content = String::new();
        let _ = resp.read_to_string(&mut content);
        Ok(content)
    };

    let backoff = ExponentialBackoff::default();
    backoff::retry(backoff, op)
}

fn main() {
    match fetch_url("https::///wrong URL") {
        Ok(_) => println!("Successfully fetched"),
        Err(err) => panic!("Failed to fetch: {}", err),
    }
}
