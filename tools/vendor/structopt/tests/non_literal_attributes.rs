// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::clap::AppSettings;
use structopt::StructOpt;

pub const DISPLAY_ORDER: usize = 2;

// Check if the global settings compile
#[derive(StructOpt, Debug, PartialEq, Eq)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
struct Opt {
    #[structopt(
        long = "x",
        display_order = DISPLAY_ORDER,
        next_line_help = true,
        default_value = "0",
        require_equals = true
    )]
    x: i32,

    #[structopt(short = "l", long = "level", aliases = &["set-level", "lvl"])]
    level: String,

    #[structopt(long("values"))]
    values: Vec<i32>,

    #[structopt(name = "FILE", requires_if("FILE", "values"))]
    files: Vec<String>,
}

#[test]
fn test_slice() {
    assert_eq!(
        Opt {
            x: 0,
            level: "1".to_string(),
            files: Vec::new(),
            values: vec![],
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-l", "1"]))
    );
    assert_eq!(
        Opt {
            x: 0,
            level: "1".to_string(),
            files: Vec::new(),
            values: vec![],
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--level", "1"]))
    );
    assert_eq!(
        Opt {
            x: 0,
            level: "1".to_string(),
            files: Vec::new(),
            values: vec![],
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--set-level", "1"]))
    );
    assert_eq!(
        Opt {
            x: 0,
            level: "1".to_string(),
            files: Vec::new(),
            values: vec![],
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--lvl", "1"]))
    );
}

#[test]
fn test_multi_args() {
    assert_eq!(
        Opt {
            x: 0,
            level: "1".to_string(),
            files: vec!["file".to_string()],
            values: vec![],
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-l", "1", "file"]))
    );
    assert_eq!(
        Opt {
            x: 0,
            level: "1".to_string(),
            files: vec!["FILE".to_string()],
            values: vec![1],
        },
        Opt::from_clap(
            &Opt::clap().get_matches_from(&["test", "-l", "1", "--values", "1", "--", "FILE"]),
        )
    );
}

#[test]
fn test_multi_args_fail() {
    let result = Opt::clap().get_matches_from_safe(&["test", "-l", "1", "--", "FILE"]);
    assert!(result.is_err());
}

#[test]
fn test_bool() {
    assert_eq!(
        Opt {
            x: 1,
            level: "1".to_string(),
            files: vec![],
            values: vec![],
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-l", "1", "--x=1"]))
    );
    let result = Opt::clap().get_matches_from_safe(&["test", "-l", "1", "--x", "1"]);
    assert!(result.is_err());
}

fn parse_hex(input: &str) -> Result<u64, std::num::ParseIntError> {
    u64::from_str_radix(input, 16)
}

#[derive(StructOpt, PartialEq, Debug)]
struct HexOpt {
    #[structopt(short = "n", parse(try_from_str = parse_hex))]
    number: u64,
}

#[test]
fn test_parse_hex_function_path() {
    assert_eq!(
        HexOpt { number: 5 },
        HexOpt::from_clap(&HexOpt::clap().get_matches_from(&["test", "-n", "5"]))
    );
    assert_eq!(
        HexOpt { number: 0xabcdef },
        HexOpt::from_clap(&HexOpt::clap().get_matches_from(&["test", "-n", "abcdef"]))
    );

    let err = HexOpt::clap()
        .get_matches_from_safe(&["test", "-n", "gg"])
        .unwrap_err();
    assert!(
        err.message.contains("invalid digit found in string"),
        "{}",
        err
    );
}
