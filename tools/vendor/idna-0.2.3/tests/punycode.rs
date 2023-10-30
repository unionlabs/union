// Copyright 2013 The rust-url developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::test::TestFn;
use idna::punycode::{decode, encode_str};
use serde_json::map::Map;
use serde_json::Value;
use std::str::FromStr;

fn one_test(decoded: &str, encoded: &str) {
    match decode(encoded) {
        None => panic!("Decoding {} failed.", encoded),
        Some(result) => {
            let result = result.into_iter().collect::<String>();
            assert!(
                result == decoded,
                "Incorrect decoding of \"{}\":\n   \"{}\"\n!= \"{}\"\n",
                encoded,
                result,
                decoded
            )
        }
    }

    match encode_str(decoded) {
        None => panic!("Encoding {} failed.", decoded),
        Some(result) => assert!(
            result == encoded,
            "Incorrect encoding of \"{}\":\n   \"{}\"\n!= \"{}\"\n",
            decoded,
            result,
            encoded
        ),
    }
}

fn get_string<'a>(map: &'a Map<String, Value>, key: &str) -> &'a str {
    match map.get(&key.to_string()) {
        Some(&Value::String(ref s)) => s,
        None => "",
        _ => panic!(),
    }
}

pub fn collect_tests<F: FnMut(String, TestFn)>(add_test: &mut F) {
    match Value::from_str(include_str!("punycode_tests.json")) {
        Ok(Value::Array(tests)) => {
            for (i, test) in tests.into_iter().enumerate() {
                match test {
                    Value::Object(o) => {
                        let test_name = {
                            let desc = get_string(&o, "description");
                            if desc.is_empty() {
                                format!("Punycode {}", i + 1)
                            } else {
                                format!("Punycode {}: {}", i + 1, desc)
                            }
                        };
                        add_test(
                            test_name,
                            TestFn::dyn_test_fn(move || {
                                one_test(get_string(&o, "decoded"), get_string(&o, "encoded"))
                            }),
                        )
                    }
                    _ => panic!(),
                }
            }
        }
        other => panic!("{:?}", other),
    }
}
