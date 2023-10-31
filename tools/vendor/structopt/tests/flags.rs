// Copyright 2018 Guillaume Pinot (@TeXitoi) <texitoi@texitoi.eu>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use structopt::StructOpt;

#[test]
fn unique_flag() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, long)]
        alice: bool,
    }

    assert_eq!(
        Opt { alice: false },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert_eq!(
        Opt { alice: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a"]))
    );
    assert_eq!(
        Opt { alice: true },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--alice"]))
    );
    assert!(Opt::clap().get_matches_from_safe(&["test", "-i"]).is_err());
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a", "foo"])
        .is_err());
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a", "-a"])
        .is_err());
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a", "--alice"])
        .is_err());
}

#[test]
fn multiple_flag() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, long, parse(from_occurrences))]
        alice: u64,
        #[structopt(short, long, parse(from_occurrences))]
        bob: u8,
    }

    assert_eq!(
        Opt { alice: 0, bob: 0 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert_eq!(
        Opt { alice: 1, bob: 0 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a"]))
    );
    assert_eq!(
        Opt { alice: 2, bob: 0 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "-a"]))
    );
    assert_eq!(
        Opt { alice: 2, bob: 2 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a", "--alice", "-bb"]))
    );
    assert_eq!(
        Opt { alice: 3, bob: 1 },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-aaa", "--bob"]))
    );
    assert!(Opt::clap().get_matches_from_safe(&["test", "-i"]).is_err());
    assert!(Opt::clap()
        .get_matches_from_safe(&["test", "-a", "foo"])
        .is_err());
}

fn parse_from_flag(b: bool) -> std::sync::atomic::AtomicBool {
    std::sync::atomic::AtomicBool::new(b)
}

#[test]
fn non_bool_flags() {
    #[derive(StructOpt, Debug)]
    struct Opt {
        #[structopt(short, long, parse(from_flag = parse_from_flag))]
        alice: std::sync::atomic::AtomicBool,
        #[structopt(short, long, parse(from_flag))]
        bob: std::sync::atomic::AtomicBool,
    }

    let falsey = Opt::from_clap(&Opt::clap().get_matches_from(&["test"]));
    assert!(!falsey.alice.load(std::sync::atomic::Ordering::Relaxed));
    assert!(!falsey.bob.load(std::sync::atomic::Ordering::Relaxed));

    let alice = Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a"]));
    assert!(alice.alice.load(std::sync::atomic::Ordering::Relaxed));
    assert!(!alice.bob.load(std::sync::atomic::Ordering::Relaxed));

    let bob = Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-b"]));
    assert!(!bob.alice.load(std::sync::atomic::Ordering::Relaxed));
    assert!(bob.bob.load(std::sync::atomic::Ordering::Relaxed));

    let both = Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-b", "-a"]));
    assert!(both.alice.load(std::sync::atomic::Ordering::Relaxed));
    assert!(both.bob.load(std::sync::atomic::Ordering::Relaxed));
}

#[test]
fn combined_flags() {
    #[derive(StructOpt, PartialEq, Debug)]
    struct Opt {
        #[structopt(short, long)]
        alice: bool,
        #[structopt(short, long, parse(from_occurrences))]
        bob: u64,
    }

    assert_eq!(
        Opt {
            alice: false,
            bob: 0
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test"]))
    );
    assert_eq!(
        Opt {
            alice: true,
            bob: 0
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a"]))
    );
    assert_eq!(
        Opt {
            alice: true,
            bob: 0
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-a"]))
    );
    assert_eq!(
        Opt {
            alice: false,
            bob: 1
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-b"]))
    );
    assert_eq!(
        Opt {
            alice: true,
            bob: 1
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "--alice", "--bob"]))
    );
    assert_eq!(
        Opt {
            alice: true,
            bob: 4
        },
        Opt::from_clap(&Opt::clap().get_matches_from(&["test", "-bb", "-a", "-bb"]))
    );
}
