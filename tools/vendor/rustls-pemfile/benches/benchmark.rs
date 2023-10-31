use criterion::{criterion_group, criterion_main, Criterion};
use std::io::BufReader;

#[test]
fn test_certs() {}

fn parse_cert() {
    let data = include_bytes!("../tests/data/certificate.chain.pem");
    let mut reader = BufReader::new(&data[..]);

    assert_eq!(
        rustls_pemfile::certs(&mut reader)
            .unwrap()
            .len(),
        3
    );
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse cert chain", |b| b.iter(|| parse_cert()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
