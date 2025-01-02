mod wrapper;

use std::iter::Peekable;
use std::str::CharIndices;

pub use wrapper::{IteratorConvertion, PeekableWrapper};

#[derive(Debug)]
pub struct LexStream<'inp> {
	iter: <Self as PeekableWrapper>::Iterator,
	source: &'inp str,
	file: &'inp str
}

impl<'inp> LexStream<'inp> {
	pub fn new(source: &'inp str, file: &'inp str) -> Self {
		let iter = source.char_indices().peekable();
		Self { iter, source, file }
	}

	pub fn source(&self) -> &'inp str { self.source }

	pub fn file(&self) -> &'inp str { self.file }

	pub fn position(&mut self) -> usize {
		let source_len = self.source.len();

		self.peek().map_or(source_len, |(idx, _)| *idx)
	}
}

impl IteratorConvertion<(usize, char)> for LexStream<'_> {
	type Output = char;

	fn convert_item(input: &(usize, char)) -> Self::Output { input.1 }
}

impl<'inp> PeekableWrapper for LexStream<'inp> {
	type Iterator = Peekable<CharIndices<'inp>>;

	fn next(&mut self) -> Option<<Self::Iterator as Iterator>::Item> { self.iter.next() }

	fn peek(&mut self) -> Option<&<Self::Iterator as Iterator>::Item> { self.iter.peek() }
}
