use backoff::{retry, Error, ExponentialBackoff};

use std::io::Read;

fn fetch_url(url: &str) -> Result<String, Error<reqwest::Error>> {
    let op = || {
        println!("Fetching {}", url);
        let mut resp = reqwest::blocking::get(url)?;

        let mut content = String::new();
        let _ = resp.read_to_string(&mut content);
        Ok(content)
    };

    let backoff = ExponentialBackoff::default();
    retry(backoff, op)
}

fn main() {
    match fetch_url("https://www.rust-lang.org") {
        Ok(_) => println!("Successfully fetched"),
        Err(err) => panic!("Failed to fetch: {}", err),
    }
}
