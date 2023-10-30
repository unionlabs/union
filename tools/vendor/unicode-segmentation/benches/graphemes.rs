use criterion::{black_box, criterion_group, criterion_main, Criterion};
use unicode_segmentation;

use std::fs;
use unicode_segmentation::UnicodeSegmentation;

fn graphemes(c: &mut Criterion, lang: &str, path: &str) {
    let text = fs::read_to_string(path).unwrap();

    c.bench_function(&format!("graphemes_{}", lang), |bench| {
        bench.iter(|| {
            for g in UnicodeSegmentation::graphemes(black_box(&*text), true) {
                black_box(g);
            }
        })
    });
}

fn graphemes_arabic(c: &mut Criterion) {
    graphemes(c, "arabic", "benches/texts/arabic.txt");
}

fn graphemes_english(c: &mut Criterion) {
    graphemes(c, "english", "benches/texts/english.txt");
}

fn graphemes_hindi(c: &mut Criterion) {
    graphemes(c, "hindi", "benches/texts/hindi.txt");
}

fn graphemes_japanese(c: &mut Criterion) {
    graphemes(c, "japanese", "benches/texts/japanese.txt");
}

fn graphemes_korean(c: &mut Criterion) {
    graphemes(c, "korean", "benches/texts/korean.txt");
}

fn graphemes_mandarin(c: &mut Criterion) {
    graphemes(c, "mandarin", "benches/texts/mandarin.txt");
}

fn graphemes_russian(c: &mut Criterion) {
    graphemes(c, "russian", "benches/texts/russian.txt");
}

fn graphemes_source_code(c: &mut Criterion) {
    graphemes(c, "source_code", "benches/texts/source_code.txt");
}

criterion_group!(
    benches,
    graphemes_arabic,
    graphemes_english,
    graphemes_hindi,
    graphemes_japanese,
    graphemes_korean,
    graphemes_mandarin,
    graphemes_russian,
    graphemes_source_code,
);

criterion_main!(benches);
