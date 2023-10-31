use criterion::{black_box, Criterion};

fn wincon(c: &mut Criterion) {
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
        group.bench_function("wincon_bytes", |b| {
            b.iter(|| {
                let mut state = anstream::adapter::WinconBytes::new();
                let stripped = state.extract_next(content).collect::<Vec<_>>();

                black_box(stripped)
            })
        });
    }
}

criterion::criterion_group!(benches, wincon);
criterion::criterion_main!(benches);
