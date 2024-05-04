use ariadne::{Label, Report, ReportKind, Source};

use std::fmt;
use std::io;

pub type Position = std::ops::Range<usize>;
pub type Span<'a> = (&'a str, Position);

pub trait AriadneCompatible {
	fn message(&self) -> String;
	fn labels<'a>(&'a self, file: &'a str) -> Vec<Label<Span>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrettyError<Kind: AriadneCompatible> {
	pub file: String,
	pub source: String,
	pub position: Position,
	pub kind: Kind
}

impl<Kind: AriadneCompatible> PrettyError<Kind> {
	pub fn new(source: String, file: String, position: Position, kind: Kind) -> Self {
		Self { source, file, position, kind }
	}

	fn create_report(&self) -> Report<Span> {
		let mut report = Report::build(
			ReportKind::Error,
			&self.source[..],
			self.position.start
		).with_message(self.kind.message());

		report.add_labels(
			self.kind.labels(&self.file)
		);

		report.finish()
	}

	fn get_cache(&self) -> (&str, Source<&str>) {
		(&self.file[..], Source::from(&self.source[..]))
	}

	pub fn print(&self) {
		let report = self.create_report();
		let cache = self.get_cache();

		report
			.print(cache)
			.unwrap();
	}
}

impl<Kind: AriadneCompatible> fmt::Display for PrettyError<Kind> {
	fn fmt<'fmtref>(&self, f: &'fmtref mut fmt::Formatter<'_>) -> fmt::Result {
		use io::Write;

		let report = self.create_report();
		let cache = self.get_cache();
		let mut writer = FormatterWriter::<'fmtref, '_>::new(f);
		
		report
			.write(cache, &mut writer)
			.map_err(move |_| fmt::Error)?;

		writer
			.flush()
			.map_err(|_| fmt::Error)
	}
}

struct FormatterWriter<'fmtref, 'fmt> {
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

impl<'fmtref, 'fmt> io::Write for FormatterWriter<'fmtref, 'fmt> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.buffer.extend_from_slice(buf);
		Ok(buf.len())
	}

	fn flush(&mut self) -> io::Result<()> {
		use std::str::from_utf8;
		use io::{Error, ErrorKind};

		match from_utf8(&self.buffer) {
			Ok(s) => self
				.formatter
				.write_str(s)
				.map_err(|e| Error::new(ErrorKind::Other, e)),
			
			Err(_) => Err(
				Error::new(
					ErrorKind::Other,
					"Failed to convert buffer to string"
				)
			),
		}?;

		self.buffer.clear();
		Ok(())
	}
}