#![feature(test)]
extern crate test;

#[bench]
fn bench_64(b: &mut test::Bencher) {
    let mut buf = [0u8; 64];
    b.iter(|| {
        getrandom::getrandom(&mut buf[..]).unwrap();
        test::black_box(&buf);
    });
    b.bytes = buf.len() as u64;
}

#[bench]
fn bench_65536(b: &mut test::Bencher) {
    let mut buf = [0u8; 65536];
    b.iter(|| {
        getrandom::getrandom(&mut buf[..]).unwrap();
        test::black_box(&buf);
    });
    b.bytes = buf.len() as u64;
}
