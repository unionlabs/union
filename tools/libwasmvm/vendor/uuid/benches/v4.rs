#![cfg(feature = "v4")]
#![feature(test)]
extern crate test;

use test::Bencher;
use uuid::Uuid;

#[bench]
fn new_v4(b: &mut Bencher) {
    b.iter(|| Uuid::new_v4());
}
