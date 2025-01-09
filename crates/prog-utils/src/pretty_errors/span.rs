use std::fmt::{self, Debug};

#[derive(Clone, Copy, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct Span<'src> {
	source: &'src str,
	file: &'src str,
	position: Position
}

impl<'src> Span<'src> {
	pub fn new(source: &'src str, file: &'src str, position: Position) -> Self {
		assert!(position.end() <= source.len(), "Span exceeds source length");
		Self::new_unchecked(source, file, position)
	}

	pub fn new_unchecked(source: &'src str, file: &'src str, position: Position) -> Self {
		Self {
			source,
			file,
			position
		}
	}

	pub fn source(&self) -> &'src str { self.source }

	pub fn file(&self) -> &'src str { self.file }

	pub fn value(&self) -> &'src str {
		&self.source[self.position().start()..self.position().end()]
	}

	pub fn position(&self) -> Position { self.position }
}

impl<'src> ariadne::Span for Span<'src> {
	type SourceId = &'src str;

	// This is not a mistake, this function is supposed to return
	// the *source identifier* of the span, not the actual source.
	#[allow(clippy::misnamed_getters)]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
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
