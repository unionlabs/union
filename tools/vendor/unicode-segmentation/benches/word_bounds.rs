#[macro_use]
extern crate bencher;
extern crate unicode_segmentation;

use bencher::Bencher;
use std::fs;
use unicode_segmentation::UnicodeSegmentation;

fn word_bounds(bench: &mut Bencher, path: &str) {
    let text = fs::read_to_string(path).unwrap();
    bench.iter(|| {
        for w in text.split_word_bounds() {
            bencher::black_box(w);
        }
    });

    bench.bytes = text.len() as u64;
}

fn word_bounds_arabic(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/arabic.txt");
}

fn word_bounds_english(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/english.txt");
}

fn word_bounds_hindi(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/hindi.txt");
}

fn word_bounds_japanese(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/japanese.txt");
}

fn word_bounds_korean(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/korean.txt");
}

fn word_bounds_mandarin(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/mandarin.txt");
}

fn word_bounds_russian(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/russian.txt");
}

fn word_bounds_source_code(bench: &mut Bencher) {
    word_bounds(bench, "benches/texts/source_code.txt");
}

benchmark_group!(
    benches,
    word_bounds_arabic,
    word_bounds_english,
    word_bounds_hindi,
    word_bounds_japanese,
    word_bounds_korean,
    word_bounds_mandarin,
    word_bounds_russian,
    word_bounds_source_code,
);

benchmark_main!(benches);
