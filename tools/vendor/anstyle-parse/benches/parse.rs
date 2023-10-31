use criterion::{black_box, Criterion};

use anstyle_parse::*;

struct BenchDispatcher;
impl Perform for BenchDispatcher {
    fn print(&mut self, c: char) {
        black_box(c);
    }

    fn execute(&mut self, byte: u8) {
        black_box(byte);
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: u8) {
        black_box((params, intermediates, ignore, c));
    }

    fn put(&mut self, byte: u8) {
        black_box(byte);
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        black_box((params, bell_terminated));
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], ignore: bool, c: u8) {
        black_box((params, intermediates, ignore, c));
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        black_box((intermediates, ignore, byte));
    }
}

#[derive(Default)]
struct Strip(String);
impl Strip {
    fn with_capacity(capacity: usize) -> Self {
        Self(String::with_capacity(capacity))
    }
}
impl Perform for Strip {
    fn print(&mut self, c: char) {
        self.0.push(c);
    }

    fn execute(&mut self, byte: u8) {
        if byte.is_ascii_whitespace() {
            self.0.push(byte as char);
        }
    }
}

fn strip_str(content: &str) -> String {
    use anstyle_parse::state::state_change;
    use anstyle_parse::state::Action;
    use anstyle_parse::state::State;

    #[inline]
    fn is_utf8_continuation(b: u8) -> bool {
        matches!(b, 0x80..=0xbf)
    }

    #[inline]
    fn is_printable(action: Action, byte: u8) -> bool {
        action == Action::Print
                    || action == Action::BeginUtf8
                    // since we know the input is valid UTF-8, the only thing  we can do with
                    // continuations is to print them
                    || is_utf8_continuation(byte)
                    || (action == Action::Execute && byte.is_ascii_whitespace())
    }

    let mut stripped = Vec::with_capacity(content.len());

    let mut bytes = content.as_bytes();
    while !bytes.is_empty() {
        let offset = bytes.iter().copied().position(|b| {
            let (_next_state, action) = state_change(State::Ground, b);
            !is_printable(action, b)
        });
        let (printable, next) = bytes.split_at(offset.unwrap_or(bytes.len()));
        stripped.extend(printable);
        bytes = next;

        let mut state = State::Ground;
        let offset = bytes.iter().copied().position(|b| {
            let (next_state, action) = state_change(state, b);
            if next_state != State::Anywhere {
                state = next_state;
            }
            is_printable(action, b)
        });
        let (_, next) = bytes.split_at(offset.unwrap_or(bytes.len()));
        bytes = next;
    }

    String::from_utf8(stripped).unwrap()
}

fn parse(c: &mut Criterion) {
    for (name, content) in [
        #[cfg(feature = "utf8")]
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
            let mut parser = Parser::<DefaultCharAccumulator>::new();
            for byte in content.as_bytes() {
                parser.advance(&mut stripped, *byte);
            }
            assert_eq!(stripped.0, strip_str(content));
        }

        let mut group = c.benchmark_group(name);
        group.bench_function("advance", |b| {
            b.iter(|| {
                let mut dispatcher = BenchDispatcher;
                let mut parser = Parser::<DefaultCharAccumulator>::new();

                for byte in content {
                    parser.advance(&mut dispatcher, *byte);
                }
            })
        });
        group.bench_function("advance_strip", |b| {
            b.iter(|| {
                let mut stripped = Strip::with_capacity(content.len());
                let mut parser = Parser::<DefaultCharAccumulator>::new();

                for byte in content {
                    parser.advance(&mut stripped, *byte);
                }

                black_box(stripped.0)
            })
        });
        group.bench_function("state_change", |b| {
            b.iter(|| {
                let mut state = anstyle_parse::state::State::default();
                for byte in content {
                    let (next_state, action) = anstyle_parse::state::state_change(state, *byte);
                    state = next_state;
                    black_box(action);
                }
            })
        });
        if let Ok(content) = std::str::from_utf8(content) {
            group.bench_function("state_change_strip_str", |b| {
                b.iter(|| {
                    let stripped = strip_str(content);

                    black_box(stripped)
                })
            });
        }
    }
}

criterion::criterion_group!(benches, parse);
criterion::criterion_main!(benches);
