use std::fmt::{self, Debug};

use ariadne::Span as _;

#[derive(Clone, Copy, PartialEq, Hash)]
pub struct Span<'inp> {
	source: &'inp str,
	position: Position
}

impl<'inp> Span<'inp> {
	pub fn new(source: &'inp str, position: Position) -> Self {
		assert!(position.end() <= source.len(), "Span exceeds source length");
		Self::new_unchecked(source, position)
	}

	pub fn new_unchecked(source: &'inp str, position: Position) -> Self {
		Self { source, position }
	}

	pub fn value(&self) -> &'inp str { &self.source[self.start()..self.end()] }
}

impl<'inp> ariadne::Span for Span<'inp> {
	type SourceId = &'inp str;

	fn source(&self) -> &Self::SourceId { &self.source }

	fn start(&self) -> usize { self.position.start() }

	fn end(&self) -> usize { self.position.end() }
}

impl Debug for Span<'_> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut s = f.debug_struct("Span");
		s.field("position", &self.position);
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
