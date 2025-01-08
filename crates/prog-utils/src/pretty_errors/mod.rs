mod formatter_writer;
mod span;

use formatter_writer::FormatterWriter;
pub use span::{Position, Span};

use std::{fmt, io};

use ariadne::{Label, Report, ReportKind, Source};

#[cfg(feature = "serde")]
pub trait PrettyErrorKind: Clone + fmt::Debug + AriadneCompatible + serde::Serialize {}
#[cfg(not(feature = "serde"))]
pub trait PrettyErrorKind: Clone + fmt::Debug + AriadneCompatible {}

pub trait AriadneCompatible {
	fn message(&self) -> String;
	fn labels(self, span: Span) -> Vec<Label<Span>>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrettyError<Kind: PrettyErrorKind> {
	pub source: Box<str>,
	pub file: Box<str>,
	pub position: Position,
	pub kind: Kind
}

impl<Kind: PrettyErrorKind> PrettyError<Kind> {
	pub fn new<S, F>(source: S, file: F, position: Position, kind: Kind) -> Self
	where
		S: Into<Box<str>>,
		F: Into<Box<str>>
	{
		let source = source.into();
		let file = file.into();

		Self {
			source,
			file,
			position,
			kind
		}
	}

	pub fn new_unspanned(kind: Kind) -> Self {
		Self {
			source: "".into(),
			file: "".into(),
			position: Position::new(0, 0),
			kind
		}
	}

	fn create_report(&self) -> Report<Span> {
		let span = Span::new(&self.source, &self.file, self.position);
		let message = self.kind.message();

		let mut report = Report::build(ReportKind::Error, span).with_message(message);

		report.add_labels(self.kind.clone().labels(span));
		report.finish()
	}

	fn get_cache(&self) -> (&str, Source<&str>) { (&self.file[..], Source::from(&self.source[..])) }

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
		s.serialize_field("source", &self.source)?;
		s.serialize_field("file", &self.file)?;
		s.serialize_field("position", &self.position)?;
		s.serialize_field("kind", &self.kind)?;

		s.end()
	}
}

impl<Kind: PrettyErrorKind> std::error::Error for PrettyError<Kind> {}
