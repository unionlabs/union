#[macro_use]
extern crate bencher;
extern crate idna;

use bencher::{black_box, Bencher};
use idna::Config;

fn to_unicode_puny_label(bench: &mut Bencher) {
    let encoded = "abc.xn--mgbcm";
    let config = Config::default();
    bench.iter(|| config.to_unicode(black_box(encoded)));
}

fn to_unicode_ascii(bench: &mut Bencher) {
    let encoded = "example.com";
    let config = Config::default();
    bench.iter(|| config.to_unicode(black_box(encoded)));
}

fn to_unicode_merged_label(bench: &mut Bencher) {
    let encoded = "Beispiel.xn--vermgensberater-ctb";
    let config = Config::default();
    bench.iter(|| config.to_unicode(black_box(encoded)));
}

fn to_ascii_puny_label(bench: &mut Bencher) {
    let encoded = "abc.ابج";
    let config = Config::default();
    bench.iter(|| config.to_ascii(black_box(encoded)));
}

fn to_ascii_simple(bench: &mut Bencher) {
    let encoded = "example.com";
    let config = Config::default();
    bench.iter(|| config.to_ascii(black_box(encoded)));
}

fn to_ascii_merged(bench: &mut Bencher) {
    let encoded = "beispiel.vermögensberater";
    let config = Config::default();
    bench.iter(|| config.to_ascii(black_box(encoded)));
}

benchmark_group!(
    benches,
    to_unicode_puny_label,
    to_unicode_ascii,
    to_unicode_merged_label,
    to_ascii_puny_label,
    to_ascii_simple,
    to_ascii_merged,
);
benchmark_main!(benches);
