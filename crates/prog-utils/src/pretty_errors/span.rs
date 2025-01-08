use std::fmt::{self, Debug};

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct Span<'inp> {
	source: &'inp str,
	file: &'inp str,
	position: Position
}

impl<'inp> Span<'inp> {
	pub fn new(source: &'inp str, file: &'inp str, position: Position) -> Self {
		assert!(position.end() <= source.len(), "Span exceeds source length");
		Self::new_unchecked(source, file, position)
	}

	pub fn new_unchecked(source: &'inp str, file: &'inp str, position: Position) -> Self {
		Self {
			source,
			file,
			position
		}
	}

	pub fn source(&self) -> &'inp str { self.source }

	pub fn file(&self) -> &'inp str { self.file }

	pub fn value(&self) -> &'inp str {
		&self.source[self.position().start()..self.position().end()]
	}

	pub fn position(&self) -> Position { self.position }
}

impl<'inp> ariadne::Span for Span<'inp> {
	type SourceId = &'inp str;

	fn source(&self) -> &Self::SourceId { &self.file }

	fn start(&self) -> usize { self.position.start() }

	fn end(&self) -> usize { self.position.end() }
}

impl Debug for Span<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Span");
		s.field("position", &self.position);
		s.field("value", &self.value());
		s.finish_non_exhaustive()
	}
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub struct Position {
	start: usize,
	end: usize
}

impl Position {
	pub fn new(start: usize, end: usize) -> Self {
		assert!(start <= end, "Position was provided backwards");
		Self { start, end }
	}

	pub fn start(&self) -> usize { self.start }

	pub fn end(&self) -> usize { self.end }
}

impl ariadne::Span for Position {
	type SourceId = ();

	fn source(&self) -> &Self::SourceId { &() }

	fn start(&self) -> usize { self.start }

	fn end(&self) -> usize { self.end }
}

impl From<std::ops::Range<usize>> for Position {
	fn from(value: std::ops::Range<usize>) -> Self {
		Self {
			start: value.start,
			end: value.end
		}
	}
}

#[cfg(feature = "serde")]
impl serde::Serialize for Position {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeStruct;

		let mut s = serializer.serialize_struct("Position", 2)?;
		s.serialize_field("start", &self.start)?;
		s.serialize_field("end", &self.end)?;

		s.end()
	}
}
