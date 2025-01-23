use std::fmt::{self, Write};

pub struct Formatter<'a> {
	buffer: &'a mut dyn Write
}

impl<'a> Formatter<'a> {
	pub fn new(buffer: &'a mut dyn Write) -> Self { Self { buffer } }
}

impl Write for Formatter<'_> {
	fn write_str(&mut self, s: &str) -> fmt::Result { self.buffer.write_str(s) }
}
