#![feature(test)]
extern crate test;

use blake2::{Blake2b512, Blake2s256};
use digest::bench_update;
use test::Bencher;

bench_update!(
    Blake2b512::default();
    blake2b512_10 10;
    blake2b512_100 100;
    blake2b512_1000 1000;
    blake2b512_10000 10000;
);

bench_update!(
    Blake2s256::default();
    blake2s256_10 10;
    blake2s256_100 100;
    blake2s256_1000 1000;
    blake2s256_10000 10000;
);
