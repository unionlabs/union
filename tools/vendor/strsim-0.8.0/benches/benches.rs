//! Benchmarks for strsim.

#![feature(test)]

extern crate strsim;

mod benches {
    use super::*;

    extern crate test;
    use self::test::Bencher;

    #[bench]
    fn bench_hamming(bencher: &mut Bencher) {
        let a = "ACAAGATGCCATTGTCCCCCGGCCTCCTGCTGCTGCTGCTCTCCGGGG";
        let b = "CCTGGAGGGTGGCCCCACCGGCCGAGACAGCGAGCATATGCAGGAAGC";
        bencher.iter(|| {
            strsim::hamming(&a, &b).unwrap();
        })
    }

    #[bench]
    fn bench_jaro(bencher: &mut Bencher) {
        let a = "Philosopher Friedrich Nietzsche";
        let b = "Philosopher Jean-Paul Sartre";
        bencher.iter(|| {
            strsim::jaro(&a, &b);
        })
    }

    #[bench]
    fn bench_jaro_winkler(bencher: &mut Bencher) {
        let a = "Philosopher Friedrich Nietzsche";
        let b = "Philosopher Jean-Paul Sartre";
        bencher.iter(|| {
            strsim::jaro_winkler(&a, &b);
        })
    }

    #[bench]
    fn bench_levenshtein(bencher: &mut Bencher) {
        let a = "Philosopher Friedrich Nietzsche";
        let b = "Philosopher Jean-Paul Sartre";
        bencher.iter(|| {
            strsim::levenshtein(&a, &b);
        })
    }

    #[bench]
    fn bench_normalized_levenshtein(bencher: &mut Bencher) {
        let a = "Philosopher Friedrich Nietzsche";
        let b = "Philosopher Jean-Paul Sartre";
        bencher.iter(|| {
            strsim::normalized_levenshtein(&a, &b);
        })
    }

    #[bench]
    fn bench_osa_distance(bencher: &mut Bencher) {
        let a = "Philosopher Friedrich Nietzsche";
        let b = "Philosopher Jean-Paul Sartre";
        bencher.iter(|| {
            strsim::osa_distance(&a, &b);
        })
    }

    #[bench]
    fn bench_damerau_levenshtein(bencher: &mut Bencher) {
        let a = "Philosopher Friedrich Nietzsche";
        let b = "Philosopher Jean-Paul Sartre";
        bencher.iter(|| {
            strsim::damerau_levenshtein(&a, &b);
        })
    }

    #[bench]
    fn bench_normalized_damerau_levenshtein(bencher: &mut Bencher) {
        let a = "Philosopher Friedrich Nietzsche";
        let b = "Philosopher Jean-Paul Sartre";
        bencher.iter(|| {
            strsim::normalized_damerau_levenshtein(&a, &b);
        })
    }
}
