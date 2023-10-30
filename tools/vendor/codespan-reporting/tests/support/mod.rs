use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::files::Files;
use codespan_reporting::term::{emit, Config};
use termcolor::{Buffer, WriteColor};

mod color_buffer;

use self::color_buffer::ColorBuffer;

pub struct TestData<'files, F: Files<'files>> {
    pub files: F,
    pub diagnostics: Vec<Diagnostic<F::FileId>>,
}

impl<'files, F: Files<'files>> TestData<'files, F> {
    fn emit<W: WriteColor>(&'files self, mut writer: W, config: &Config) -> W {
        for diagnostic in &self.diagnostics {
            emit(&mut writer, config, &self.files, &diagnostic).unwrap();
        }
        writer
    }

    pub fn emit_color(&'files self, config: &Config) -> String {
        self.emit(ColorBuffer::new(), &config).into_string()
    }

    pub fn emit_no_color(&'files self, config: &Config) -> String {
        let buffer = self.emit(Buffer::no_color(), &config);
        String::from_utf8_lossy(buffer.as_slice()).into_owned()
    }
}
