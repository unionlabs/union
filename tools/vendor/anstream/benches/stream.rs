use std::io::Write as _;

use criterion::{black_box, Criterion};

fn stream(c: &mut Criterion) {
    for (name, content) in [
        ("demo.vte", &include_bytes!("../tests/demo.vte")[..]),
        ("rg_help.vte", &include_bytes!("../tests/rg_help.vte")[..]),
        ("rg_linus.vte", &include_bytes!("../tests/rg_linus.vte")[..]),
        (
            "state_changes",
            &b"\x1b]2;X\x1b\\ \x1b[0m \x1bP0@\x1b\\"[..],
        ),
    ] {
        let mut group = c.benchmark_group(name);
        group.bench_function("nop", |b| {
            b.iter(|| {
                let buffer = anstream::Buffer::with_capacity(content.len());
                let mut stream = buffer;

                stream.write_all(content).unwrap();

                black_box(stream)
            })
        });
        group.bench_function("StripStream", |b| {
            b.iter(|| {
                let buffer = anstream::Buffer::with_capacity(content.len());
                let mut stream = anstream::StripStream::new(buffer);

                stream.write_all(content).unwrap();

                black_box(stream)
            })
        });
        #[cfg(all(windows, feature = "wincon"))]
        group.bench_function("WinconStream", |b| {
            b.iter(|| {
                let buffer = anstream::Buffer::with_capacity(content.len());
                let mut stream =
                    anstream::WinconStream::new(anstyle_wincon::Console::new(buffer).unwrap());

                stream.write_all(content).unwrap();

                black_box(stream)
            })
        });
        group.bench_function("AutoStream::always_ansi", |b| {
            b.iter(|| {
                let buffer = anstream::Buffer::with_capacity(content.len());
                let mut stream = anstream::AutoStream::always_ansi(buffer);

                stream.write_all(content).unwrap();

                black_box(stream)
            })
        });
        group.bench_function("AutoStream::always", |b| {
            b.iter(|| {
                let buffer = anstream::Buffer::with_capacity(content.len());
                let mut stream = anstream::AutoStream::always(buffer);

                stream.write_all(content).unwrap();

                black_box(stream)
            })
        });
        group.bench_function("AutoStream::never", |b| {
            b.iter(|| {
                let buffer = anstream::Buffer::with_capacity(content.len());
                let mut stream = anstream::AutoStream::never(buffer);

                stream.write_all(content).unwrap();

                black_box(stream)
            })
        });
    }
}

criterion::criterion_group!(benches, stream);
criterion::criterion_main!(benches);
