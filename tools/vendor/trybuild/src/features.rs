use serde::de::{self, Deserialize, DeserializeOwned, Deserializer};
use serde_derive::Deserialize;
use std::env;
use std::error::Error;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

pub fn find() -> Option<Vec<String>> {
    try_find().ok()
}

struct Ignored;

impl<E: Error> From<E> for Ignored {
    fn from(_error: E) -> Self {
        Ignored
    }
}

#[derive(Deserialize)]
struct Build {
    #[serde(deserialize_with = "from_json")]
    features: Vec<String>,
}

fn try_find() -> Result<Vec<String>, Ignored> {
    // This will look something like:
    //   /path/to/crate_name/target/debug/deps/test_name-HASH
    let test_binary = env::args_os().next().ok_or(Ignored)?;

    // The hash at the end is ascii so not lossy, rest of conversion doesn't
    // matter.
    let test_binary_lossy = test_binary.to_string_lossy();
    let hash_range = if cfg!(windows) {
        // Trim ".exe" from the binary name for windows.
        test_binary_lossy.len() - 21..test_binary_lossy.len() - 4
    } else {
        test_binary_lossy.len() - 17..test_binary_lossy.len()
    };
    let hash = test_binary_lossy.get(hash_range).ok_or(Ignored)?;
    if !hash.starts_with('-') || !hash[1..].bytes().all(is_lower_hex_digit) {
        return Err(Ignored);
    }

    let binary_path = PathBuf::from(&test_binary);

    // Feature selection is saved in:
    //   /path/to/crate_name/target/debug/.fingerprint/*-HASH/*-HASH.json
    let up = binary_path
        .parent()
        .ok_or(Ignored)?
        .parent()
        .ok_or(Ignored)?;
    let fingerprint_dir = up.join(".fingerprint");
    if !fingerprint_dir.is_dir() {
        return Err(Ignored);
    }

    let mut hash_matches = Vec::new();
    for entry in fingerprint_dir.read_dir()? {
        let entry = entry?;
        let is_dir = entry.file_type()?.is_dir();
        let matching_hash = entry.file_name().to_string_lossy().ends_with(hash);
        if is_dir && matching_hash {
            hash_matches.push(entry.path());
        }
    }

    if hash_matches.len() != 1 {
        return Err(Ignored);
    }

    let mut json_matches = Vec::new();
    for entry in hash_matches[0].read_dir()? {
        let entry = entry?;
        let is_file = entry.file_type()?.is_file();
        let is_json = entry.path().extension() == Some(OsStr::new("json"));
        if is_file && is_json {
            json_matches.push(entry.path());
        }
    }

    if json_matches.len() != 1 {
        return Err(Ignored);
    }

    let build_json = fs::read_to_string(&json_matches[0])?;
    let build: Build = serde_json::from_str(&build_json)?;
    Ok(build.features)
}

fn is_lower_hex_digit(byte: u8) -> bool {
    byte >= b'0' && byte <= b'9' || byte >= b'a' && byte <= b'f'
}

fn from_json<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: DeserializeOwned,
    D: Deserializer<'de>,
{
    let json = String::deserialize(deserializer)?;
    serde_json::from_str(&json).map_err(de::Error::custom)
}
