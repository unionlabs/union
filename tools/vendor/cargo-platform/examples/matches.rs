//! This example demonstrates how to filter a Platform based on the current
//! host target.

use cargo_platform::{Cfg, Platform};
use std::process::Command;
use std::str::FromStr;

static EXAMPLES: &[&str] = &[
    "cfg(windows)",
    "cfg(unix)",
    "cfg(target_os=\"macos\")",
    "cfg(target_os=\"linux\")",
    "cfg(any(target_arch=\"x86\", target_arch=\"x86_64\"))",
];

fn main() {
    let target = get_target();
    let cfgs = get_cfgs();
    println!("host target={} cfgs:", target);
    for cfg in &cfgs {
        println!("  {}", cfg);
    }
    let mut examples: Vec<&str> = EXAMPLES.iter().copied().collect();
    examples.push(target.as_str());
    for example in examples {
        let p = Platform::from_str(example).unwrap();
        println!("{:?} matches: {:?}", example, p.matches(&target, &cfgs));
    }
}

fn get_target() -> String {
    let output = Command::new("rustc")
        .arg("-Vv")
        .output()
        .expect("rustc failed to run");
    let stdout = String::from_utf8(output.stdout).unwrap();
    for line in stdout.lines() {
        if line.starts_with("host: ") {
            return String::from(&line[6..]);
        }
    }
    panic!("Failed to find host: {}", stdout);
}

fn get_cfgs() -> Vec<Cfg> {
    let output = Command::new("rustc")
        .arg("--print=cfg")
        .output()
        .expect("rustc failed to run");
    let stdout = String::from_utf8(output.stdout).unwrap();
    stdout
        .lines()
        .map(|line| Cfg::from_str(line).unwrap())
        .collect()
}
