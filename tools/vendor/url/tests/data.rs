// Copyright 2013-2014 The rust-url developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Data-driven tests

use std::str::FromStr;

use serde_json::Value;
use url::{quirks, Url};

#[test]
fn urltestdata() {
    // Copied from https://github.com/web-platform-tests/wpt/blob/master/url/
    let mut json = Value::from_str(include_str!("urltestdata.json"))
        .expect("JSON parse error in urltestdata.json");

    let mut passed = true;
    let mut skip_next = false;
    for entry in json.as_array_mut().unwrap() {
        if entry.is_string() {
            if entry.as_str().unwrap() == "skip next" {
                skip_next = true;
            }
            continue; // ignore comments
        }

        if skip_next {
            skip_next = false;
            continue;
        }

        let maybe_base = entry
            .take_key("base")
            .expect("missing base key")
            .maybe_string();
        let input = entry.take_string("input");
        let failure = entry.take_key("failure").is_some();

        let res = if let Some(base) = maybe_base {
            let base = match Url::parse(&base) {
                Ok(base) => base,
                Err(_) if failure => continue,
                Err(message) => {
                    eprint_failure(
                        format!("  failed: error parsing base {:?}: {}", base, message),
                        &format!("parse base for {:?}", input),
                        None,
                    );
                    passed = false;
                    continue;
                }
            };
            base.join(&input)
        } else {
            Url::parse(&input)
        };

        let url = match (res, failure) {
            (Ok(url), false) => url,
            (Err(_), true) => continue,
            (Err(message), false) => {
                eprint_failure(
                    format!("  failed: {}", message),
                    &format!("parse URL for {:?}", input),
                    None,
                );
                passed = false;
                continue;
            }
            (Ok(_), true) => {
                eprint_failure(
                    format!("  failed: expected parse error for URL {:?}", input),
                    &format!("parse URL for {:?}", input),
                    None,
                );
                passed = false;
                continue;
            }
        };

        passed &= check_invariants(&url, &format!("invariants for {:?}", input), None);

        for &attr in ATTRIBS {
            passed &= test_eq_eprint(
                entry.take_string(attr),
                get(&url, attr),
                &format!("{:?} - {}", input, attr),
                None,
            );
        }

        if let Some(expected_origin) = entry.take_key("origin").map(|s| s.string()) {
            passed &= test_eq_eprint(
                expected_origin,
                &quirks::origin(&url),
                &format!("origin for {:?}", input),
                None,
            );
        }
    }

    assert!(passed)
}

#[test]
fn setters_tests() {
    let mut json = Value::from_str(include_str!("setters_tests.json"))
        .expect("JSON parse error in setters_tests.json");

    let mut passed = true;
    for &attr in ATTRIBS {
        if attr == "href" {
            continue;
        }

        let mut tests = json.take_key(attr).unwrap();
        for mut test in tests.as_array_mut().unwrap().drain(..) {
            let comment = test.take_key("comment").map(|s| s.string());
            {
                if let Some(comment) = comment.as_ref() {
                    if comment.starts_with("IDNA Nontransitional_Processing") {
                        continue;
                    }
                }
            }
            let href = test.take_string("href");
            let new_value = test.take_string("new_value");
            let name = format!("{:?}.{} = {:?}", href, attr, new_value);
            let mut expected = test.take_key("expected").unwrap();

            let mut url = Url::parse(&href).unwrap();
            let comment_ref = comment.as_deref();
            passed &= check_invariants(&url, &name, comment_ref);
            set(&mut url, attr, &new_value);

            for attr in ATTRIBS {
                if let Some(value) = expected.take_key(attr) {
                    passed &= test_eq_eprint(value.string(), get(&url, attr), &name, comment_ref);
                };
            }

            passed &= check_invariants(&url, &name, comment_ref);
        }
    }

    assert!(passed);
}

fn check_invariants(url: &Url, name: &str, comment: Option<&str>) -> bool {
    let mut passed = true;
    if let Err(e) = url.check_invariants() {
        passed = false;
        eprint_failure(
            format!("  failed: invariants checked -> {:?}", e),
            name,
            comment,
        );
    }

    #[cfg(feature = "serde")]
    {
        let bytes = serde_json::to_vec(url).unwrap();
        let new_url: Url = serde_json::from_slice(&bytes).unwrap();
        passed &= test_eq_eprint(url.to_string(), &new_url.to_string(), name, comment);
    }

    passed
}

trait JsonExt {
    fn take_key(&mut self, key: &str) -> Option<Value>;
    fn string(self) -> String;
    fn maybe_string(self) -> Option<String>;
    fn take_string(&mut self, key: &str) -> String;
}

impl JsonExt for Value {
    fn take_key(&mut self, key: &str) -> Option<Value> {
        self.as_object_mut().unwrap().remove(key)
    }

    fn string(self) -> String {
        self.maybe_string().expect("")
    }

    fn maybe_string(self) -> Option<String> {
        match self {
            Value::String(s) => Some(s),
            Value::Null => None,
            _ => panic!("Not a Value::String or Value::Null"),
        }
    }

    fn take_string(&mut self, key: &str) -> String {
        self.take_key(key).unwrap().string()
    }
}

fn get<'a>(url: &'a Url, attr: &str) -> &'a str {
    match attr {
        "href" => quirks::href(url),
        "protocol" => quirks::protocol(url),
        "username" => quirks::username(url),
        "password" => quirks::password(url),
        "hostname" => quirks::hostname(url),
        "host" => quirks::host(url),
        "port" => quirks::port(url),
        "pathname" => quirks::pathname(url),
        "search" => quirks::search(url),
        "hash" => quirks::hash(url),
        _ => unreachable!(),
    }
}

#[allow(clippy::unit_arg)]
fn set(url: &mut Url, attr: &str, new: &str) {
    let _ = match attr {
        "protocol" => quirks::set_protocol(url, new),
        "username" => quirks::set_username(url, new),
        "password" => quirks::set_password(url, new),
        "hostname" => quirks::set_hostname(url, new),
        "host" => quirks::set_host(url, new),
        "port" => quirks::set_port(url, new),
        "pathname" => Ok(quirks::set_pathname(url, new)),
        "search" => Ok(quirks::set_search(url, new)),
        "hash" => Ok(quirks::set_hash(url, new)),
        _ => unreachable!(),
    };
}

fn test_eq_eprint(expected: String, actual: &str, name: &str, comment: Option<&str>) -> bool {
    if expected == actual {
        return true;
    }
    eprint_failure(
        format!("expected: {}\n  actual: {}", expected, actual),
        name,
        comment,
    );
    false
}

fn eprint_failure(err: String, name: &str, comment: Option<&str>) {
    eprintln!("    test: {}\n{}", name, err);
    if let Some(comment) = comment {
        eprintln!("{}\n", comment);
    } else {
        eprintln!();
    }
}

const ATTRIBS: &[&str] = &[
    "href", "protocol", "username", "password", "host", "hostname", "port", "pathname", "search",
    "hash",
];
