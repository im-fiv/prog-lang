use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Debug)]
pub struct LexStream<'src> {
	iter: Peekable<CharIndices<'src>>,
	source: &'src str,
	file: &'src str
}

impl<'src> LexStream<'src> {
	pub fn new(source: &'src str, file: &'src str) -> Self {
		let iter = source.char_indices().peekable();
		Self { iter, source, file }
	}

	pub fn source(&self) -> &'src str { self.source }

	pub fn file(&self) -> &'src str { self.file }

	pub fn position(&mut self) -> usize {
		let source_len = self.source.len();

		self.peek().map_or(source_len, |(idx, _)| *idx)
	}

	pub fn peek(&mut self) -> Option<&(usize, char)> { self.iter.peek() }

	/// Checks whether the next value matches a predicate without advancing the iterator.
	pub fn peek_matches<F>(&mut self, pred: F, consume: bool) -> bool
	where
		F: FnOnce(&char) -> bool
	{
		let matches = self.peek().is_some_and(|(_, c)| pred(c));

		if matches && consume {
			self.next();
		}

		matches
	}

	/// Checks whether the next value matches an exact value without advancing the iterator.
	pub fn peek_matches_exact(&mut self, pred: char, consume: bool) -> bool {
		self.peek_matches(|c| *c == pred, consume)
	}

	/// Advances the iterator until the predicate returns `false`.
	///
	/// Return value of `true` indicates that a match was found before the iterator was exhausted.
	pub fn next_while<F>(&mut self, pred: F) -> bool
	where
		F: Fn(&(usize, char)) -> bool
	{
		while let Some(item) = self.peek() {
			if !pred(item) {
				return true;
			}

			self.next();
		}

		false
	}

	/// Advances the iterator until the value exactly matches the predicate.
	///
	/// Return value of `true` indicates that a match was found before the iterator was exhausted.
	pub fn next_while_exact(&mut self, pred: char, consume: bool) -> bool {
		let matches = self.next_while(|(_, c)| *c != pred);

		if matches && consume {
			self.next();
		}

		matches
	}
}

impl Iterator for LexStream<'_> {
	type Item = (usize, char);

	fn next(&mut self) -> Option<Self::Item> { self.iter.next() }
}
