#![cfg(feature = "full")]

use tokio::test;

#[test]
async fn test_macro_can_be_used_via_use() {
    tokio::spawn(async {}).await.unwrap();
}

#[tokio::test]
async fn test_macro_is_resilient_to_shadowing() {
    tokio::spawn(async {}).await.unwrap();
}

// https://github.com/tokio-rs/tokio/issues/3403
#[rustfmt::skip] // this `rustfmt::skip` is necessary because unused_braces does not warn if the block contains newline.
#[tokio::main]
pub async fn unused_braces_main() { println!("hello") }
#[rustfmt::skip] // this `rustfmt::skip` is necessary because unused_braces does not warn if the block contains newline.
#[tokio::test]
async fn unused_braces_test() { assert_eq!(1 + 1, 2) }

// https://github.com/tokio-rs/tokio/pull/3766#issuecomment-835508651
#[std::prelude::v1::test]
fn trait_method() {
    trait A {
        fn f(self);
    }
    impl A for () {
        #[tokio::main]
        async fn f(self) {}
    }
    ().f()
}

// https://github.com/tokio-rs/tokio/issues/4175
#[tokio::main]
pub async fn issue_4175_main_1() -> ! {
    panic!();
}
#[tokio::main]
pub async fn issue_4175_main_2() -> std::io::Result<()> {
    panic!();
}
#[allow(unreachable_code)]
#[tokio::test]
pub async fn issue_4175_test() -> std::io::Result<()> {
    return Ok(());
    panic!();
}

// https://github.com/tokio-rs/tokio/issues/4175
pub mod clippy_semicolon_if_nothing_returned {
    #![deny(clippy::semicolon_if_nothing_returned)]

    #[tokio::main]
    pub async fn local() {
        let _x = ();
    }
    #[tokio::main]
    pub async fn item() {
        fn _f() {}
    }
    #[tokio::main]
    pub async fn semi() {
        panic!();
    }
    #[tokio::main]
    pub async fn empty() {
        // To trigger clippy::semicolon_if_nothing_returned lint, the block needs to contain newline.
    }
}
