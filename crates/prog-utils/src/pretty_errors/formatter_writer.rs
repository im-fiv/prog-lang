use std::{fmt, io};

pub(crate) struct FormatterWriter<'fmtref, 'fmt> {
	buffer: Vec<u8>,
	formatter: &'fmtref mut fmt::Formatter<'fmt>
}

impl<'fmtref, 'fmt> FormatterWriter<'fmtref, 'fmt> {
	pub fn new(formatter: &'fmtref mut fmt::Formatter<'fmt>) -> Self {
		Self {
			buffer: vec![],
			formatter
		}
	}
}

impl io::Write for FormatterWriter<'_, '_> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.buffer.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		use std::str::from_utf8;

		use io::{Error, ErrorKind};

		match from_utf8(&self.buffer) {
			Ok(s) => {
				self.formatter
					.write_str(s)
					.map_err(|e| Error::new(ErrorKind::Other, e))
			}

			Err(_) => {
				Err(Error::new(
					ErrorKind::Other,
					"Failed to convert buffer to string"
				))
			}
		}?;

		self.buffer.clear();
		Ok(())
	}
}
