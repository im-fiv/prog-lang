mod formatter_writer;
mod span;

use formatter_writer::FormatterWriter;
pub use span::{Position, Span};

use std::{fmt, io};

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

/// Initializes a color generator with a specific initial state.
pub fn color_generator() -> ColorGenerator {
	// List of alternative cherry-picked states:
	// [3689, 1234, 283]

	const STATE: [u16; 3] = [25595, 293, 9239];
	const MIN_BRIGHTNESS: f32 = 0.5;

	ColorGenerator::from_state(STATE, MIN_BRIGHTNESS)
}

#[cfg(feature = "serde")]
pub trait PrettyErrorKind<'s>: AriadneCompatible<'s> + serde::Serialize + fmt::Debug {}
#[cfg(not(feature = "serde"))]
pub trait PrettyErrorKind<'s>: AriadneCompatible<'s> + fmt::Debug {}

pub trait AriadneCompatible<'s> {
	fn message(&self) -> &'static str;

	fn note(&self) -> Option<&str> { None }

	fn labels(&self, span: Span<'s>) -> Vec<Label<Span<'s>>>;

	fn create_report(&self, span: Span<'s>) -> Report<Span<'s>> {
		let mut report = Report::build(ReportKind::Error, span);
		report.set_message(self.message());
		report.add_labels(self.labels(span));

		if let Some(note) = self.note() {
			report.set_note(note);
		}

		report.finish()
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrettyError<'s, Kind: PrettyErrorKind<'s>> {
	pub span: Span<'s>,
	pub kind: Kind
}

impl<'s, Kind: PrettyErrorKind<'s>> PrettyError<'s, Kind> {
	pub fn new(span: Span<'s>, kind: Kind) -> Self { Self { span, kind } }

	pub fn new_unspanned(kind: Kind) -> Self {
		let position = Position::new(0, 0);
		let span = Span::new_unchecked("", "", position);

		Self::new(span, kind)
	}

	pub fn from_raw_parts(source: &'s str, file: &'s str, position: Position, kind: Kind) -> Self {
		let span = Span::new(source, file, position);
		Self::new(span, kind)
	}

	fn create_report(&self) -> Report<Span<'s>> { self.kind.create_report(self.span) }

	fn get_cache(&self) -> (&str, Source<&str>) {
		(self.span.file(), Source::from(self.span.source()))
	}

	pub fn eprint(&self) {
		let report = self.create_report();
		let cache = self.get_cache();

		report
			.eprint(cache)
			.unwrap_or_else(|_| panic!("Failed to print error to stderr"));
	}
}

impl<'s, Kind: PrettyErrorKind<'s>> fmt::Display for PrettyError<'s, Kind> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use io::Write;

		let cache = self.get_cache();
		let mut writer = FormatterWriter::new(f);

		self.create_report()
			.write(cache, &mut writer)
			.map_err(|_| fmt::Error)?;

		writer.flush().map_err(|_| fmt::Error)
	}
}

#[cfg(feature = "serde")]
impl<'s, Kind: PrettyErrorKind<'s>> serde::Serialize for PrettyError<'s, Kind> {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeStruct;

		let mut s = serializer.serialize_struct("PrettyError", 3)?;
		s.serialize_field("message", &self.kind.message())?;
		s.serialize_field("span", &self.span)?;
		s.serialize_field("kind", &self.kind)?;

		s.end()
	}
}

impl<'s, Kind: PrettyErrorKind<'s>> std::error::Error for PrettyError<'s, Kind> {}
