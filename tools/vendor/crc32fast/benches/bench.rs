#[macro_use]
extern crate bencher;
extern crate crc32fast;
extern crate rand;

use bencher::Bencher;
use crc32fast::Hasher;
use rand::Rng;

fn bench(b: &mut Bencher, size: usize, hasher_init: Hasher) {
    let mut bytes = vec![0u8; size];
    rand::thread_rng().fill(&mut bytes[..]);

    b.iter(|| {
        let mut hasher = hasher_init.clone();
        hasher.update(&bytes);
        bencher::black_box(hasher.finalize())
    });

    b.bytes = size as u64;
}

fn bench_kilobyte_baseline(b: &mut Bencher) {
    bench(b, 1024, Hasher::internal_new_baseline(0, 0))
}

fn bench_kilobyte_specialized(b: &mut Bencher) {
    bench(b, 1024, Hasher::internal_new_specialized(0, 0).unwrap())
}

fn bench_megabyte_baseline(b: &mut Bencher) {
    bench(b, 1024 * 1024, Hasher::internal_new_baseline(0, 0))
}

fn bench_megabyte_specialized(b: &mut Bencher) {
    bench(
        b,
        1024 * 1024,
        Hasher::internal_new_specialized(0, 0).unwrap(),
    )
}

benchmark_group!(
    bench_baseline,
    bench_kilobyte_baseline,
    bench_megabyte_baseline
);
benchmark_group!(
    bench_specialized,
    bench_kilobyte_specialized,
    bench_megabyte_specialized
);
benchmark_main!(bench_baseline, bench_specialized);
