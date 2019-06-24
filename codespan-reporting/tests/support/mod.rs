use codespan::Files;
use codespan_reporting::diagnostic::Diagnostic;
use codespan_reporting::term::{emit, Config};
use termcolor::{Buffer, WriteColor};

mod color_buffer;

use self::color_buffer::ColorBuffer;

pub struct TestData {
    pub files: Files,
    pub diagnostics: Vec<Diagnostic>,
}

impl TestData {
    fn emit<W: WriteColor>(&self, mut writer: W, config: &Config) -> W {
        for diagnostic in &self.diagnostics {
            emit(&mut writer, config, &self.files, &diagnostic).unwrap();
        }
        writer
    }

    pub fn emit_color(&self, config: &Config) -> String {
        self.emit(ColorBuffer::new(), &config).into_string()
    }

    pub fn emit_no_color(&self, config: &Config) -> String {
        let buffer = self.emit(Buffer::no_color(), &config);
        String::from_utf8_lossy(buffer.as_slice()).into_owned()
    }
}
