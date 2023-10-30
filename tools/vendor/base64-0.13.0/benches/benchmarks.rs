extern crate base64;
#[macro_use]
extern crate criterion;
extern crate rand;

use base64::display;
use base64::{
    decode, decode_config_buf, decode_config_slice, encode, encode_config_buf, encode_config_slice,
    write, Config,
};

use criterion::{black_box, Bencher, Criterion, ParameterizedBenchmark, Throughput};
use rand::{FromEntropy, Rng};
use std::io::{self, Read, Write};

const TEST_CONFIG: Config = base64::STANDARD;

fn do_decode_bench(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size * 3 / 4);
    fill(&mut v);
    let encoded = encode(&v);

    b.iter(|| {
        let orig = decode(&encoded);
        black_box(&orig);
    });
}

fn do_decode_bench_reuse_buf(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size * 3 / 4);
    fill(&mut v);
    let encoded = encode(&v);

    let mut buf = Vec::new();
    b.iter(|| {
        decode_config_buf(&encoded, TEST_CONFIG, &mut buf).unwrap();
        black_box(&buf);
        buf.clear();
    });
}

fn do_decode_bench_slice(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size * 3 / 4);
    fill(&mut v);
    let encoded = encode(&v);

    let mut buf = Vec::new();
    buf.resize(size, 0);
    b.iter(|| {
        decode_config_slice(&encoded, TEST_CONFIG, &mut buf).unwrap();
        black_box(&buf);
    });
}

fn do_decode_bench_stream(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size * 3 / 4);
    fill(&mut v);
    let encoded = encode(&v);

    let mut buf = Vec::new();
    buf.resize(size, 0);
    buf.truncate(0);

    b.iter(|| {
        let mut cursor = io::Cursor::new(&encoded[..]);
        let mut decoder = base64::read::DecoderReader::new(&mut cursor, TEST_CONFIG);
        decoder.read_to_end(&mut buf).unwrap();
        buf.clear();
        black_box(&buf);
    });
}

fn do_encode_bench(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    fill(&mut v);
    b.iter(|| {
        let e = encode(&v);
        black_box(&e);
    });
}

fn do_encode_bench_display(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    fill(&mut v);
    b.iter(|| {
        let e = format!("{}", display::Base64Display::with_config(&v, TEST_CONFIG));
        black_box(&e);
    });
}

fn do_encode_bench_reuse_buf(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    fill(&mut v);
    let mut buf = String::new();
    b.iter(|| {
        encode_config_buf(&v, TEST_CONFIG, &mut buf);
        buf.clear();
    });
}

fn do_encode_bench_slice(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    fill(&mut v);
    let mut buf = Vec::new();
    // conservative estimate of encoded size
    buf.resize(v.len() * 2, 0);
    b.iter(|| {
        encode_config_slice(&v, TEST_CONFIG, &mut buf);
    });
}

fn do_encode_bench_stream(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    fill(&mut v);
    let mut buf = Vec::new();

    buf.reserve(size * 2);
    b.iter(|| {
        buf.clear();
        let mut stream_enc = write::EncoderWriter::new(&mut buf, TEST_CONFIG);
        stream_enc.write_all(&v).unwrap();
        stream_enc.flush().unwrap();
    });
}

fn do_encode_bench_string_stream(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    fill(&mut v);

    b.iter(|| {
        let mut stream_enc = write::EncoderStringWriter::new(TEST_CONFIG);
        stream_enc.write_all(&v).unwrap();
        stream_enc.flush().unwrap();
        let _ = stream_enc.into_inner();
    });
}

fn do_encode_bench_string_reuse_buf_stream(b: &mut Bencher, &size: &usize) {
    let mut v: Vec<u8> = Vec::with_capacity(size);
    fill(&mut v);

    let mut buf = String::new();
    b.iter(|| {
        buf.clear();
        let mut stream_enc = write::EncoderStringWriter::from(&mut buf, TEST_CONFIG);
        stream_enc.write_all(&v).unwrap();
        stream_enc.flush().unwrap();
        let _ = stream_enc.into_inner();
    });
}

fn fill(v: &mut Vec<u8>) {
    let cap = v.capacity();
    // weak randomness is plenty; we just want to not be completely friendly to the branch predictor
    let mut r = rand::rngs::SmallRng::from_entropy();
    while v.len() < cap {
        v.push(r.gen::<u8>());
    }
}

const BYTE_SIZES: [usize; 5] = [3, 50, 100, 500, 3 * 1024];

// Benchmarks over these byte sizes take longer so we will run fewer samples to
// keep the benchmark runtime reasonable.
const LARGE_BYTE_SIZES: [usize; 3] = [3 * 1024 * 1024, 10 * 1024 * 1024, 30 * 1024 * 1024];

fn encode_benchmarks(byte_sizes: &[usize]) -> ParameterizedBenchmark<usize> {
    ParameterizedBenchmark::new("encode", do_encode_bench, byte_sizes.iter().cloned())
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(3))
        .throughput(|s| Throughput::Bytes(*s as u64))
        .with_function("encode_display", do_encode_bench_display)
        .with_function("encode_reuse_buf", do_encode_bench_reuse_buf)
        .with_function("encode_slice", do_encode_bench_slice)
        .with_function("encode_reuse_buf_stream", do_encode_bench_stream)
        .with_function("encode_string_stream", do_encode_bench_string_stream)
        .with_function(
            "encode_string_reuse_buf_stream",
            do_encode_bench_string_reuse_buf_stream,
        )
}

fn decode_benchmarks(byte_sizes: &[usize]) -> ParameterizedBenchmark<usize> {
    ParameterizedBenchmark::new("decode", do_decode_bench, byte_sizes.iter().cloned())
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(3))
        .throughput(|s| Throughput::Bytes(*s as u64))
        .with_function("decode_reuse_buf", do_decode_bench_reuse_buf)
        .with_function("decode_slice", do_decode_bench_slice)
        .with_function("decode_stream", do_decode_bench_stream)
}

fn bench(c: &mut Criterion) {
    c.bench("bench_small_input", encode_benchmarks(&BYTE_SIZES[..]));

    c.bench(
        "bench_large_input",
        encode_benchmarks(&LARGE_BYTE_SIZES[..]).sample_size(10),
    );

    c.bench("bench_small_input", decode_benchmarks(&BYTE_SIZES[..]));

    c.bench(
        "bench_large_input",
        decode_benchmarks(&LARGE_BYTE_SIZES[..]).sample_size(10),
    );
}

criterion_group!(benches, bench);
criterion_main!(benches);
