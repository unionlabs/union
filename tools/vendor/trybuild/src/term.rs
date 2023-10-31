use once_cell::sync::OnceCell;
use std::io::{Result, Write};
use std::sync::{Mutex, MutexGuard, PoisonError};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream as Stream, WriteColor};

static TERM: OnceCell<Mutex<Term>> = OnceCell::new();

pub fn lock() -> MutexGuard<'static, Term> {
    TERM.get_or_init(|| Mutex::new(Term::new()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

pub fn bold() {
    lock().set_color(ColorSpec::new().set_bold(true));
}

pub fn color(color: Color) {
    lock().set_color(ColorSpec::new().set_fg(Some(color)));
}

pub fn bold_color(color: Color) {
    lock().set_color(ColorSpec::new().set_bold(true).set_fg(Some(color)));
}

pub fn reset() {
    lock().reset();
}

#[deny(unused_macros)]
macro_rules! print {
    ($($args:tt)*) => {{
        use std::io::Write;
        let _ = std::write!($crate::term::lock(), $($args)*);
    }};
}

#[deny(unused_macros)]
macro_rules! println {
    ($($args:tt)*) => {{
        use std::io::Write;
        let _ = std::writeln!($crate::term::lock(), $($args)*);
    }};
}

pub struct Term {
    spec: ColorSpec,
    stream: Stream,
    start_of_line: bool,
}

impl Term {
    fn new() -> Self {
        Term {
            spec: ColorSpec::new(),
            stream: Stream::stderr(ColorChoice::Auto),
            start_of_line: true,
        }
    }

    fn set_color(&mut self, spec: &ColorSpec) {
        if self.spec != *spec {
            self.spec = spec.clone();
            self.start_of_line = true;
        }
    }

    fn reset(&mut self) {
        self.spec = ColorSpec::new();
        let _ = self.stream.reset();
    }
}

impl Write for Term {
    // Color one line at a time because Travis does not preserve color setting
    // across output lines.
    fn write(&mut self, mut buf: &[u8]) -> Result<usize> {
        if self.spec.is_none() {
            return self.stream.write(buf);
        }

        let len = buf.len();
        while !buf.is_empty() {
            if self.start_of_line {
                let _ = self.stream.set_color(&self.spec);
            }
            match buf.iter().position(|byte| *byte == b'\n') {
                Some(line_len) => {
                    self.stream.write_all(&buf[..line_len + 1])?;
                    self.start_of_line = true;
                    buf = &buf[line_len + 1..];
                }
                None => {
                    self.stream.write_all(buf)?;
                    self.start_of_line = false;
                    break;
                }
            }
        }
        Ok(len)
    }

    fn flush(&mut self) -> Result<()> {
        self.stream.flush()
    }
}
