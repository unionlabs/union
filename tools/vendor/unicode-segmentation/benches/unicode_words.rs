#[macro_use]
extern crate bencher;
extern crate unicode_segmentation;

use bencher::Bencher;
use std::fs;
use unicode_segmentation::UnicodeSegmentation;

fn unicode_words(bench: &mut Bencher, path: &str) {
    let text = fs::read_to_string(path).unwrap();
    bench.iter(|| {
        for w in text.unicode_words() {
            bencher::black_box(w);
        }
    });

    bench.bytes = text.len() as u64;
}

fn unicode_words_arabic(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/arabic.txt");
}

fn unicode_words_english(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/english.txt");
}

fn unicode_words_hindi(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/hindi.txt");
}

fn unicode_words_japanese(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/japanese.txt");
}

fn unicode_words_korean(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/korean.txt");
}

fn unicode_words_mandarin(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/mandarin.txt");
}

fn unicode_words_russian(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/russian.txt");
}

fn unicode_words_source_code(bench: &mut Bencher) {
    unicode_words(bench, "benches/texts/source_code.txt");
}

benchmark_group!(
    benches,
    unicode_words_arabic,
    unicode_words_english,
    unicode_words_hindi,
    unicode_words_japanese,
    unicode_words_korean,
    unicode_words_mandarin,
    unicode_words_russian,
    unicode_words_source_code,
);

benchmark_main!(benches);
