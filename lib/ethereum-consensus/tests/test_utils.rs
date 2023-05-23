#![cfg(feature = "spec-tests")]

use serde::Deserialize;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[derive(Debug)]
pub enum Config {
    Minimal,
    Mainnet,
}

pub trait TestCase: fmt::Debug {
    fn should_succeed(&self) -> bool;

    fn verify_success(&self) -> bool;

    fn verify_failure(&self) -> bool;

    fn execute(&self) {
        let result = if self.should_succeed() {
            self.verify_success()
        } else {
            self.verify_failure()
        };
        assert!(result, "{self:#?}")
    }
}

pub fn load_yaml<T: for<'de> Deserialize<'de>>(path: &str) -> T {
    let mut file = File::open(path).expect("File does not exist");
    let test_case: Result<T, _> = serde_yaml::from_reader(&mut file);
    match test_case {
        Ok(test_case) => test_case,
        Err(err) => {
            let content = std::fs::read_to_string(path).unwrap();
            panic!("{err} from {content} at {path:?}")
        }
    }
}

pub fn load_snappy_ssz_bytes(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).unwrap();
    let mut data = vec![];
    file.read_to_end(&mut data).unwrap();

    let mut decoder = snap::raw::Decoder::new();
    decoder.decompress_vec(&data).unwrap()
}

pub fn load_snappy_ssz<T: ssz_rs::Deserialize>(path: &str) -> Option<T> {
    let path = Path::new(path);
    if !path.exists() {
        return None;
    }
    let buffer = load_snappy_ssz_bytes(path);

    let result = <T as ssz_rs::Deserialize>::deserialize(&buffer);

    result.ok()
}
