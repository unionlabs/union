use criterion::{black_box, Criterion};

#[derive(Default)]
struct Strip(String);
impl Strip {
    fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }
}
impl anstyle_parse::Perform for Strip {
    fn print(&mut self, c: char) {
        self.0.push(c);
    }

    fn execute(&mut self, byte: u8) {
        if byte.is_ascii_whitespace() {
            self.0.push(byte as char);
        }
    }
}

fn strip(c: &mut Criterion) {
    for (name, content) in [
        ("demo.vte", &include_bytes!("../tests/demo.vte")[..]),
        ("rg_help.vte", &include_bytes!("../tests/rg_help.vte")[..]),
        ("rg_linus.vte", &include_bytes!("../tests/rg_linus.vte")[..]),
        (
            "state_changes",
            &b"\x1b]2;X\x1b\\ \x1b[0m \x1bP0@\x1b\\"[..],
        ),
    ] {
        // Make sure the comparison is fair
        if let Ok(content) = std::str::from_utf8(content) {
            let mut stripped = Strip::with_capacity(content.len());
            let mut parser = anstyle_parse::Parser::<anstyle_parse::DefaultCharAccumulator>::new();
            for byte in content.as_bytes() {
                parser.advance(&mut stripped, *byte);
            }
            assert_eq!(
                stripped.0,
                anstream::adapter::strip_str(content).to_string()
            );
            assert_eq!(
                stripped.0,
                String::from_utf8(anstream::adapter::strip_bytes(content.as_bytes()).into_vec())
                    .unwrap()
            );
        }

        let mut group = c.benchmark_group(name);
        group.bench_function("advance_strip", |b| {
            b.iter(|| {
                let mut stripped = Strip::with_capacity(content.len());
                let mut parser =
                    anstyle_parse::Parser::<anstyle_parse::DefaultCharAccumulator>::new();

                for byte in content {
                    parser.advance(&mut stripped, *byte);
                }

                black_box(stripped.0)
            })
        });
        group.bench_function("strip_ansi_escapes", |b| {
            b.iter(|| {
                let stripped = strip_ansi_escapes::strip(content).unwrap();

                black_box(stripped)
            })
        });
        if let Ok(content) = std::str::from_utf8(content) {
            group.bench_function("strip_str", |b| {
                b.iter(|| {
                    let stripped = anstream::adapter::strip_str(content).to_string();

                    black_box(stripped)
                })
            });
            group.bench_function("StripStr", |b| {
                b.iter(|| {
                    let mut stripped = String::with_capacity(content.len());
                    let mut state = anstream::adapter::StripStr::new();
                    for printable in state.strip_next(content) {
                        stripped.push_str(printable);
                    }

                    black_box(stripped)
                })
            });
        }
        group.bench_function("strip_bytes", |b| {
            b.iter(|| {
                let stripped = anstream::adapter::strip_bytes(content).into_vec();

                black_box(stripped)
            })
        });
    }
}

criterion::criterion_group!(benches, strip);
criterion::criterion_main!(benches);
