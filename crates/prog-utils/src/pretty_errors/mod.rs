mod formatter_writer;
mod span;

use formatter_writer::FormatterWriter;
pub use span::{Span, Position};

use std::{fmt, io};
use ariadne::{Label, Report, ReportKind, Source};

#[cfg(feature = "serde")]
pub trait PrettyErrorKind: Clone + AriadneCompatible + serde::Serialize {}
#[cfg(not(feature = "serde"))]
pub trait PrettyErrorKind: Clone + AriadneCompatible {}

pub trait AriadneCompatible {
	fn message(&self) -> String;
	fn labels(self, file: &str, position: Position) -> Vec<Label<Span>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrettyError<Kind: PrettyErrorKind> {
	pub file: String,
	pub source: String,
	pub position: Position,
	pub kind: Kind
}

impl<Kind: PrettyErrorKind> PrettyError<Kind> {
	pub fn new(source: String, file: String, position: Position, kind: Kind) -> Self {
		Self {
			source,
			file,
			position,
			kind
		}
	}

	fn create_report(&self) -> Report<Span> {
		let span = Span::new(&self.source[..], self.position);
		let message = self.kind.message();

		let mut report =
			Report::build(ReportKind::Error, span)
				.with_message(message);

		report.add_labels(self.kind.clone().labels(&self.file, self.position));
		report.finish()
	}

	fn get_cache(&self) -> (&str, Source<&str>) {
		(&self.file[..], Source::from(&self.source[..]))
	}

	pub fn eprint(&self) {
		let report = self.create_report();
		let cache = self.get_cache();

		report
			.eprint(cache)
			.unwrap_or_else(|_| panic!("Failed to print error to stderr"));
	}
}

impl<Kind: PrettyErrorKind> fmt::Display for PrettyError<Kind> {
	fn fmt<'fmtref>(&self, f: &'fmtref mut fmt::Formatter<'_>) -> fmt::Result {
		use io::Write;

		let report = self.create_report();
		let cache = self.get_cache();
		let mut writer = FormatterWriter::<'fmtref, '_>::new(f);

		report.write(cache, &mut writer).map_err(|_| fmt::Error)?;

		writer.flush().map_err(|_| fmt::Error)
	}
}

#[cfg(feature = "serde")]
impl<Kind: PrettyErrorKind> serde::Serialize for PrettyError<Kind> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeStruct;

		let mut s = serializer.serialize_struct("PrettyError", 5)?;
		s.serialize_field("message", &self.kind.message())?;
		s.serialize_field("file", &self.file)?;
		s.serialize_field("source", &self.source)?;
		s.serialize_field("position", &self.position)?;
		s.serialize_field("kind", &self.kind)?;

		s.end()
	}
}
